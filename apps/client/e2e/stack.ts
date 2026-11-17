import { SQL, type Subprocess } from 'bun';
import { createHash, randomBytes } from 'node:crypto';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import type { Stack } from './pages';

const root = resolve(import.meta.dir, '../../..');
const api = resolve(root, process.env.CARGO_TARGET_DIR ?? 'target', 'debug/api');
const clientDir = resolve(root, 'apps/client/build');
const local = 'postgres://postgres:postgres@127.0.0.1:5673/postgres?sslmode=disable';
const admin = process.env.E2E_ADMIN_URL ?? process.env.TEST_DATABASE_URL ?? local;
const database = 'diapo_e2e';

/** The space Alice and Bob share. */
export const E2E_SPACE = 'e2e';

function freePort(): number {
	const probe = Bun.serve({ port: 0, fetch: () => new Response() });
	const port = probe.port ?? 0;
	probe.stop(true);
	return port;
}

async function freshDatabase(): Promise<string> {
	const sql = new SQL(admin);
	await sql.unsafe(`DROP DATABASE IF EXISTS ${database} WITH (FORCE)`);
	await sql.unsafe(`CREATE DATABASE ${database}`);
	await sql.close();
	const url = new URL(admin);
	url.pathname = `/${database}`;
	return url.toString();
}

async function ready(base: string, server: Subprocess): Promise<void> {
	for (let attempt = 0; attempt < 300; attempt += 1) {
		if (server.exitCode !== null) throw new Error(`the API exited with ${server.exitCode}`);
		const answer = await fetch(`${base}/ready`).catch(() => null);
		if (answer?.ok) return;
		await Bun.sleep(100);
	}
	throw new Error('the API did not become ready in 30 seconds');
}

/**
 * Writes one person and their browser session straight into the scratch database, the way a
 * sign-in leaves them: porte stores the SHA-256 of a `b~` token. Answers the cookie value.
 */
async function person(sql: SQL, name: string, role: string): Promise<string> {
	const token = `b~${randomBytes(32).toString('base64url')}`;
	const hash = createHash('sha256').update(token).digest('hex');
	const login = name.toLowerCase();
	const subject = `e2e-${login}`;
	const [user] = await sql`INSERT INTO porte_users (email, name) VALUES (${`${login}@e2e.test`}, ${name}) RETURNING id`;
	await sql`INSERT INTO porte_identities (user_id, provider, subject) VALUES (${user.id}, '', ${subject})`;
	await sql`INSERT INTO porte_sessions (token_hash, user_id, expires_at)
		VALUES (${hash}, ${user.id}, now() + interval '1 hour')`;
	await sql`INSERT INTO space_members (space_id, subject, email, name, role)
		VALUES (${E2E_SPACE}, ${subject}, ${`${login}@e2e.test`}, ${name}, ${role})`;
	return token;
}

/** Seeds the space `E2E_SPACE` with Alice, its owner, and Bob, a member. Answers their cookies. */
async function seed(url: string): Promise<{ alice: string; bob: string }> {
	const sql = new SQL(url);
	await sql`INSERT INTO spaces (id, name) VALUES (${E2E_SPACE}, 'Équipe e2e')`;
	const alice = await person(sql, 'Alice', 'owner');
	const bob = await person(sql, 'Bob', 'member');
	await sql.close();
	return { alice, bob };
}

/**
 * Starts the Diapo API on a fresh database and a free port, serving the built client, with two
 * signed-in people: Alice, whose cookie is `cookie`, and Bob, whose cookie is `guest`.
 * `E2E_LOG=1` shows the API's request log.
 */
export async function startStack(): Promise<Stack> {
	for (const path of [api, `${clientDir}/index.html`]) {
		if (!existsSync(path)) throw new Error(`${path} is missing: run "mise run e2e", which builds it`);
	}
	const base = `http://127.0.0.1:${freePort()}`;
	const dataDir = mkdtempSync(join(tmpdir(), 'diapo-e2e-'));
	const url = await freshDatabase();
	const env = {
		...process.env,
		DATABASE_URL: url,
		PORT: new URL(base).port,
		PUBLIC_URL: base,
		CLIENT_DIR: clientDir,
		DATA_DIR: dataDir,
		OIDC_ISSUER: '',
		RUST_LOG: process.env.RUST_LOG ?? 'warn'
	};
	const server = Bun.spawn([api], { env, stdout: process.env.E2E_LOG ? 'inherit' : 'ignore', stderr: 'inherit' });
	await ready(base, server);
	const stop = async () => {
		server.kill('SIGTERM');
		await server.exited;
		rmSync(dataDir, { recursive: true, force: true });
	};
	const { alice, bob } = await seed(url);
	return { base, cookie: alice, guest: bob, stop };
}
