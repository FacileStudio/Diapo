import type { Row } from '../row';

/** The words inside the landing demos. File names, commands and markup stay as they are. */
export const landingDemo = {
	'demo.label': [
		'Demo: text typed in the source becomes a slide, then the presentation starts.',
		'Démonstration : le texte tapé dans la source devient une diapositive, puis la présentation démarre.',
		'Demostración: el texto escrito en la fuente se convierte en una diapositiva y empieza la presentación.',
		'Demo: Im Quelltext getippter Text wird zu einer Folie, dann startet die Präsentation.'
	],
	'demo.deck': ['Q3 review', 'Bilan T3', 'Balance T3', 'Q3-Rückblick'],
	'demo.sub': [
		'What moved, what comes next.',
		'Ce qui a bougé, la suite.',
		'Lo que cambió, lo que sigue.',
		'Was sich bewegt hat, was kommt.'
	],
	'demo.heading': ['Revenue', 'Revenus', 'Ingresos', 'Umsatz'],
	'demo.first': ['Up 12%', 'En hausse de 12 %', 'Sube un 12 %', '12 % im Plus'],
	'demo.second': ['Churn down', 'Attrition en baisse', 'Menos bajas', 'Weniger Abgänge'],
	'demo.thanks': ['Thank you', 'Merci', 'Gracias', 'Danke'],
	'demo.file.title': ['One text file', 'Un seul fichier texte', 'Un solo archivo de texto', 'Eine einzige Textdatei'],
	'demo.file.body': [
		'A line of three dashes starts a new slide. Everything else is your text.',
		'Une ligne de trois tirets commence une nouvelle diapositive. Tout le reste est votre texte.',
		'Una línea de tres guiones empieza una diapositiva nueva. Todo lo demás es tu texto.',
		'Eine Zeile mit drei Strichen beginnt eine neue Folie. Alles andere ist Ihr Text.'
	],
	'demo.bulle.title': [
		'Bulle writes and edits',
		'Bulle rédige et retouche',
		'Bulle redacta y retoca',
		'Bulle schreibt und überarbeitet'
	],
	'demo.bulle.body': [
		'Ask for a slide or a rewrite, and Bulle edits the text as you watch. With a Pro or Studio plan.',
		'Demandez une diapositive ou une reformulation, Bulle modifie le texte sous vos yeux. Avec une offre Pro ou Studio.',
		'Pide una diapositiva o una reformulación y Bulle modifica el texto ante tus ojos. Con un plan Pro o Studio.',
		'Bitten Sie um eine Folie oder eine Umformulierung, Bulle ändert den Text vor Ihren Augen. Mit Pro- oder Studio-Tarif.'
	],
	'demo.bulle.ask': [
		'Add a point about churn',
		'Ajoute un point sur l’attrition',
		'Añade un punto sobre las bajas',
		'Füge einen Punkt zu den Abgängen hinzu'
	],
	'demo.bulle.done': ['Slide updated', 'Diapositive modifiée', 'Diapositiva modificada', 'Folie geändert'],
	'demo.present.title': [
		'Present from the browser',
		'Présentez depuis le navigateur',
		'Presenta desde el navegador',
		'Präsentieren Sie im Browser'
	],
	'demo.present.body': [
		'Full screen on one side. Your notes, the next slide and a timer on the other.',
		'Plein écran d’un côté. Vos notes, la diapositive suivante et un chrono de l’autre.',
		'Pantalla completa por un lado. Tus notas, la diapositiva siguiente y un cronómetro por el otro.',
		'Vollbild auf der einen Seite. Notizen, nächste Folie und Stoppuhr auf der anderen.'
	],
	'demo.present.note': [
		'Dwell on churn.',
		'Insister sur l’attrition.',
		'Insistir en las bajas.',
		'Bei den Abgängen verweilen.'
	],
	'demo.terminal.title': [
		'From the terminal too',
		'Aussi depuis le terminal',
		'También desde el terminal',
		'Auch im Terminal'
	],
	'demo.terminal.body': [
		'The diapo command creates, serves and pushes the same file from your machine.',
		'La commande diapo crée, sert et envoie le même fichier depuis votre machine.',
		'El comando diapo crea, sirve y envía el mismo archivo desde tu equipo.',
		'Der Befehl diapo erstellt, startet und überträgt dieselbe Datei auf Ihrem Rechner.'
	]
} satisfies Record<string, Row>;
