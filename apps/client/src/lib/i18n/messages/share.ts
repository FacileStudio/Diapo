import type { Row } from '../row';

/** The share dialog, the export dialog and the public viewer. */
export const share = {
	'share.title': ['Share', 'Partager', 'Compartir', 'Teilen'],
	'share.failed': [
		'The share state could not be read.',
		'L’état du partage n’a pas pu être lu.',
		'No se pudo leer el estado del enlace.',
		'Der Freigabestatus konnte nicht gelesen werden.'
	],
	'share.off': [
		'Create a link, and anyone who has it can watch this presentation in a browser, without an account. They never see your source or your notes.',
		'Créez un lien : toute personne qui l’a peut regarder cette présentation dans un navigateur, sans compte. Elle ne voit jamais votre source ni vos notes.',
		'Crea un enlace: quien lo tenga puede ver esta presentación en un navegador, sin cuenta. Nunca ve tu fuente ni tus notas.',
		'Erstellen Sie einen Link: Wer ihn hat, kann diese Präsentation im Browser ansehen, ohne Konto. Quelltext und Notizen bleiben verborgen.'
	],
	'share.create': ['Create the link', 'Créer le lien', 'Crear el enlace', 'Link erstellen'],
	'share.once': [
		'Copy this link now',
		'Copiez ce lien maintenant',
		'Copia este enlace ahora',
		'Kopieren Sie diesen Link jetzt'
	],
	'share.once.hint': [
		'It is shown only this once. Diapo does not keep it and cannot show it again. If you lose it, generate a new one.',
		'Il n’est affiché que cette fois. Diapo ne le conserve pas et ne pourra pas vous le remontrer. Si vous le perdez, générez-en un nouveau.',
		'Solo se muestra esta vez. Diapo no lo guarda y no podrá mostrarlo de nuevo. Si lo pierdes, genera uno nuevo.',
		'Er wird nur dieses eine Mal angezeigt. Diapo speichert ihn nicht und kann ihn nicht erneut zeigen. Geht er verloren, erzeugen Sie einen neuen.'
	],
	'share.link': ['Link', 'Lien', 'Enlace', 'Link'],
	'share.copy': ['Copy the link', 'Copier le lien', 'Copiar el enlace', 'Link kopieren'],
	'share.embed': ['Embed code', 'Code d’intégration', 'Código de inserción', 'Einbettungscode'],
	'share.embed.hint': [
		'Paste it into a page to show the presentation there.',
		'Collez-le dans une page pour y afficher la présentation.',
		'Pégalo en una página para mostrar allí la presentación.',
		'Fügen Sie ihn in eine Seite ein, um die Präsentation dort zu zeigen.'
	],
	'share.embed.copy': [
		'Copy the embed code',
		'Copier le code d’intégration',
		'Copiar el código de inserción',
		'Einbettungscode kopieren'
	],
	'share.on': [
		'This presentation is shared since {moment}.',
		'Cette présentation est partagée depuis le {moment}.',
		'Esta presentación está compartida desde el {moment}.',
		'Diese Präsentation ist seit {moment} freigegeben.'
	],
	'share.hidden': [
		'The link was shown when it was created and cannot be shown again. Generate a new one if you no longer have it.',
		'Le lien a été affiché à sa création et ne peut pas être remontré. Générez-en un nouveau si vous ne l’avez plus.',
		'El enlace se mostró al crearlo y no puede mostrarse de nuevo. Genera uno nuevo si ya no lo tienes.',
		'Der Link wurde bei der Erstellung angezeigt und kann nicht erneut gezeigt werden. Erzeugen Sie einen neuen, wenn Sie ihn nicht mehr haben.'
	],
	'share.rotate': ['Generate a new link', 'Générer un nouveau lien', 'Generar un enlace nuevo', 'Neuen Link erzeugen'],
	'share.rotate.title': ['Replace the link?', 'Remplacer le lien ?', '¿Reemplazar el enlace?', 'Link ersetzen?'],
	'share.rotate.hint': [
		'The current link stops working at once, wherever it was sent or embedded.',
		'Le lien actuel cesse de fonctionner tout de suite, partout où il a été envoyé ou intégré.',
		'El enlace actual deja de funcionar de inmediato, dondequiera que se haya enviado o insertado.',
		'Der aktuelle Link funktioniert sofort nicht mehr, überall, wo er verschickt oder eingebettet wurde.'
	],
	'share.revoke': ['Turn the link off', 'Désactiver le lien', 'Desactivar el enlace', 'Link deaktivieren'],
	'share.revoke.title': ['Turn the link off?', 'Désactiver le lien ?', '¿Desactivar el enlace?', 'Link deaktivieren?'],
	'share.revoke.hint': [
		'Every copy of this link stops working, for good.',
		'Toutes les copies de ce lien cessent de fonctionner, définitivement.',
		'Todas las copias de este enlace dejan de funcionar, para siempre.',
		'Alle Kopien dieses Links funktionieren dann dauerhaft nicht mehr.'
	],
	'export.title': ['Export', 'Exporter', 'Exportar', 'Exportieren'],
	'export.pptx': ['PowerPoint', 'PowerPoint', 'PowerPoint', 'PowerPoint'],
	'export.pptx.hint': [
		'An editable .pptx file. Animated HTML slides are exported as a placeholder for now.',
		'Un fichier .pptx modifiable. Les diapositives HTML animées sont exportées comme un simple repère pour l’instant.',
		'Un archivo .pptx editable. Por ahora, las diapositivas HTML animadas se exportan como un marcador.',
		'Eine bearbeitbare .pptx-Datei. Animierte HTML-Folien werden vorerst als Platzhalter exportiert.'
	],
	'export.html': ['HTML', 'HTML', 'HTML', 'HTML'],
	'export.html.hint': [
		'One file that opens offline in any browser.',
		'Un seul fichier, qui s’ouvre hors ligne dans n’importe quel navigateur.',
		'Un solo archivo que se abre sin conexión en cualquier navegador.',
		'Eine einzige Datei, die offline in jedem Browser öffnet.'
	],
	'export.pdf': ['PDF', 'PDF', 'PDF', 'PDF'],
	'export.pdf.hint': [
		'One page per slide.',
		'Une page par diapositive.',
		'Una página por diapositiva.',
		'Eine Seite pro Folie.'
	],
	'export.mp4': ['Video', 'Vidéo', 'Vídeo', 'Video'],
	'export.mp4.hint': [
		'An MP4 file of the whole presentation.',
		'Un fichier MP4 de toute la présentation.',
		'Un archivo MP4 de toda la presentación.',
		'Eine MP4-Datei der gesamten Präsentation.'
	],
	'export.start': ['Export', 'Exporter', 'Exportar', 'Exportieren'],
	'export.working': ['Preparing…', 'Préparation…', 'Preparando…', 'Wird vorbereitet…'],
	'export.download': ['Download', 'Télécharger', 'Descargar', 'Herunterladen'],
	'export.failed': [
		'The export failed.',
		'L’export a échoué.',
		'La exportación falló.',
		'Der Export ist fehlgeschlagen.'
	],
	'public.gone': [
		'This link is no longer available',
		'Ce lien n’est plus disponible',
		'Este enlace ya no está disponible',
		'Dieser Link ist nicht mehr verfügbar'
	],
	'public.gone.hint': [
		'Its owner turned it off or replaced it. Ask them for a new link.',
		'Son auteur l’a désactivé ou remplacé. Demandez-lui un nouveau lien.',
		'Su autor lo desactivó o lo reemplazó. Pídele un enlace nuevo.',
		'Der Besitzer hat ihn deaktiviert oder ersetzt. Bitten Sie um einen neuen Link.'
	],
	'public.made': ['Made with Diapo', 'Fait avec Diapo', 'Hecho con Diapo', 'Erstellt mit Diapo'],
	'public.open': [
		'Open in a new tab',
		'Ouvrir dans un nouvel onglet',
		'Abrir en una pestaña nueva',
		'In neuem Tab öffnen'
	]
} satisfies Record<string, Row>;
