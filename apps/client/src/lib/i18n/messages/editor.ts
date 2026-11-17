import type { Row } from '../row';

/** The editor around the slides: preview, files, the room's connection, the starter deck and Bulle. */
export const editor = {
	'preview.title': ['Preview', 'Aperçu', 'Vista previa', 'Vorschau'],
	'save.clean': ['Saved', 'Enregistré', 'Guardado', 'Gespeichert'],
	'save.saving': ['Saving…', 'Enregistrement…', 'Guardando…', 'Speichern…'],
	'position.scene': [
		'Scene {scene} of {scenes}',
		'Scène {scene} sur {scenes}',
		'Escena {scene} de {scenes}',
		'Szene {scene} von {scenes}'
	],
	'position.step': [
		'step {step} of {steps}',
		'étape {step} sur {steps}',
		'paso {step} de {steps}',
		'Schritt {step} von {steps}'
	],
	'step.previous': ['Previous', 'Précédent', 'Anterior', 'Zurück'],
	'step.next': ['Next', 'Suivant', 'Siguiente', 'Weiter'],
	'strip.count': [
		{ one: '{count} scene', other: '{count} scenes' },
		{ one: '{count} scène', other: '{count} scènes' },
		{ one: '{count} escena', other: '{count} escenas' },
		{ one: '{count} Szene', other: '{count} Szenen' }
	],
	'insert.label': ['Insert', 'Insérer', 'Insertar', 'Einfügen'],
	'starter.sub': [
		'One idea per slide.',
		'Une idée par diapositive.',
		'Una idea por diapositiva.',
		'Eine Idee pro Folie.'
	],
	'starter.heading': ['First topic', 'Premier sujet', 'Primer tema', 'Erstes Thema'],
	'starter.first': [
		'A point that appears on the next click',
		'Un point qui apparaît au clic suivant',
		'Un punto que aparece en el siguiente clic',
		'Ein Punkt, der beim nächsten Klick erscheint'
	],
	'starter.second': ['Then the one after', 'Puis le suivant', 'Luego el siguiente', 'Dann der nächste'],
	'starter.note': [
		'Your notes go here. They show in the presenter view, never on screen.',
		'Vos notes vont ici. Elles s’affichent dans la vue de l’orateur, jamais à l’écran.',
		'Tus notas van aquí. Se muestran en la vista del orador, nunca en pantalla.',
		'Ihre Notizen stehen hier. Sie erscheinen in der Referentenansicht, nie auf der Leinwand.'
	],
	'assets.title': ['Files', 'Fichiers', 'Archivos', 'Dateien'],
	'assets.hint': [
		'Pictures, videos and other files this presentation uses.',
		'Les images, vidéos et autres fichiers que cette présentation utilise.',
		'Las imágenes, vídeos y otros archivos que usa esta presentación.',
		'Bilder, Videos und andere Dateien, die diese Präsentation verwendet.'
	],
	'assets.drop': ['Drop files here', 'Déposez des fichiers ici', 'Suelte archivos aquí', 'Dateien hier ablegen'],
	'assets.browse': ['Browse', 'Parcourir', 'Examinar', 'Durchsuchen'],
	'assets.empty': ['No file yet.', 'Aucun fichier pour l’instant.', 'Ningún archivo por ahora.', 'Noch keine Datei.'],
	'assets.insert': [
		'Insert {name} on this slide',
		'Insérer {name} sur cette diapositive',
		'Insertar {name} en esta diapositiva',
		'{name} auf dieser Folie einfügen'
	],
	'assets.delete': ['Delete {name}', 'Supprimer {name}', 'Eliminar {name}', '{name} löschen'],
	'assets.failed': [
		'The file could not be sent.',
		'Le fichier n’a pas pu être envoyé.',
		'No se pudo enviar el archivo.',
		'Die Datei konnte nicht gesendet werden.'
	],
	'room.syncing': [
		'Opening the presentation…',
		'Ouverture de la présentation…',
		'Abriendo la presentación…',
		'Präsentation wird geöffnet…'
	],
	'room.connecting': [
		'Reconnecting. Editing resumes once the presentation is in sync.',
		'Reconnexion. L’édition reprend dès que la présentation est synchronisée.',
		'Reconectando. La edición se reanuda cuando la presentación esté sincronizada.',
		'Verbindung wird wiederhergestellt. Die Bearbeitung geht weiter, sobald die Präsentation synchron ist.'
	],
	'room.offline': [
		'You are offline. Editing resumes when the connection is back.',
		'Vous êtes hors ligne. L’édition reprend au retour de la connexion.',
		'Está sin conexión. La edición se reanuda cuando vuelva la conexión.',
		'Sie sind offline. Die Bearbeitung geht weiter, sobald die Verbindung zurück ist.'
	],
	'room.refused': [
		'The server refused a change. Reload to continue from the saved presentation.',
		'Le serveur a refusé une modification. Rechargez pour repartir de la présentation enregistrée.',
		'El servidor rechazó un cambio. Recargue para seguir desde la presentación guardada.',
		'Der Server hat eine Änderung abgelehnt. Laden Sie neu, um mit der gespeicherten Präsentation weiterzuarbeiten.'
	],
	'room.too-large': [
		'That change is too large to save. Reload to continue from the saved presentation.',
		'Cette modification est trop lourde pour être enregistrée. Rechargez pour repartir de la présentation enregistrée.',
		'Ese cambio es demasiado grande para guardarse. Recargue para seguir desde la presentación guardada.',
		'Diese Änderung ist zu groß zum Speichern. Laden Sie neu, um mit der gespeicherten Präsentation weiterzuarbeiten.'
	],
	'room.failed': [
		'The editing session stopped. Reload to continue from the saved presentation.',
		'La session d’édition s’est arrêtée. Rechargez pour repartir de la présentation enregistrée.',
		'La sesión de edición se detuvo. Recargue para seguir desde la presentación guardada.',
		'Die Bearbeitungssitzung wurde beendet. Laden Sie neu, um mit der gespeicherten Präsentation weiterzuarbeiten.'
	],
	'room.reload': ['Reload', 'Recharger', 'Recargar', 'Neu laden'],
	'bulle.ask': ['Ask {name}', 'Demander à {name}', 'Preguntar a {name}', '{name} fragen']
} satisfies Record<string, Row>;
