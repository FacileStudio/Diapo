import type { Row } from '../row';

/** The page where a person confirms or refuses an action an agent asked for. */
export const pending = {
	'pending.title': ['Agent request', 'Demande d’un agent', 'Solicitud de un agente', 'Anfrage eines Agenten'],
	'pending.subtitle': [
		'An agent wants to make a change that needs your approval.',
		'Un agent veut faire une modification qui demande votre accord.',
		'Un agente quiere hacer un cambio que requiere su aprobación.',
		'Ein Agent möchte eine Änderung vornehmen, die Ihre Zustimmung braucht.'
	],
	'pending.ready': ['Ready', 'Prête', 'Lista', 'Bereit'],
	'pending.changed': [
		'The presentation changed since the request: confirming will fail',
		'La présentation a changé depuis la demande : la confirmation échouera',
		'La presentación cambió desde la solicitud: la confirmación fallará',
		'Die Präsentation wurde seit der Anfrage geändert: Die Bestätigung schlägt fehl'
	],
	'pending.gone': [
		'The presentation no longer exists, or you no longer have access to it',
		'La présentation n’existe plus ou vous n’y avez plus accès',
		'La presentación ya no existe o ya no tiene acceso a ella',
		'Die Präsentation existiert nicht mehr, oder Sie haben keinen Zugriff mehr'
	],
	'pending.client': ['Client {client}', 'Client {client}', 'Cliente {client}', 'Client {client}'],
	'pending.expires': ['Expires {moment}', 'Expire le {moment}', 'Caduca el {moment}', 'Läuft ab am {moment}'],
	'pending.open': ['Open the presentation', 'Ouvrir la présentation', 'Abrir la presentación', 'Präsentation öffnen'],
	'pending.confirmed': ['Request confirmed', 'Demande confirmée', 'Solicitud confirmada', 'Anfrage bestätigt'],
	'pending.confirmed.hint': [
		'The presentation is in the trash, where it stays 30 days.',
		'La présentation est dans la corbeille, où elle reste 30 jours.',
		'La presentación está en la papelera, donde permanece 30 días.',
		'Die Präsentation liegt im Papierkorb und bleibt dort 30 Tage.'
	],
	'pending.dismissed': ['Request refused', 'Demande refusée', 'Solicitud rechazada', 'Anfrage abgelehnt'],
	'pending.dismissed.hint': [
		'The presentation stays as it is.',
		'La présentation reste telle quelle.',
		'La presentación queda como está.',
		'Die Präsentation bleibt, wie sie ist.'
	],
	'pending.missing': ['Request not found', 'Demande introuvable', 'Solicitud no encontrada', 'Anfrage nicht gefunden'],
	'pending.missing.hint': [
		'It was already confirmed, refused or expired, or it is not in your name.',
		'Elle a déjà été confirmée, refusée ou a expiré, ou elle n’est pas à votre nom.',
		'Ya fue confirmada, rechazada o caducó, o no está a su nombre.',
		'Sie wurde bereits bestätigt, abgelehnt oder ist abgelaufen, oder sie läuft nicht auf Ihren Namen.'
	],
	'pending.truncated': [
		'Too many open requests',
		'Trop de demandes ouvertes',
		'Demasiadas solicitudes abiertas',
		'Zu viele offene Anfragen'
	],
	'pending.truncated.hint': [
		'This one is not among the 100 most recent. Answer the most recent first, then open this link again.',
		'Celle-ci n’est pas parmi les 100 plus récentes. Répondez d’abord aux plus récentes, puis rouvrez ce lien.',
		'Esta no está entre las 100 más recientes. Responda primero a las más recientes y vuelva a abrir este enlace.',
		'Diese gehört nicht zu den 100 neuesten. Beantworten Sie zuerst die neuesten und öffnen Sie den Link erneut.'
	],
	'pending.browser': [
		'Sign in again in your browser',
		'Reconnectez-vous dans votre navigateur',
		'Vuelva a iniciar sesión en su navegador',
		'Melden Sie sich im Browser erneut an'
	],
	'pending.browser.hint': [
		'Only a session opened in your browser can confirm an agent request.',
		'Seule une session ouverte dans votre navigateur peut confirmer une demande d’agent.',
		'Solo una sesión abierta en su navegador puede confirmar una solicitud de agente.',
		'Nur eine im Browser geöffnete Sitzung kann eine Agentenanfrage bestätigen.'
	],
	'pending.signin': ['Sign in again', 'Se reconnecter', 'Volver a iniciar sesión', 'Erneut anmelden']
} satisfies Record<string, Row>;
