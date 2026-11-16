import type { Row } from '../row';

/** The shell, the deck list, the settings page and the error page. */
export const decks = {
	account: ['Account', 'Compte', 'Cuenta', 'Konto'],
	'action.cancel': ['Cancel', 'Annuler', 'Cancelar', 'Abbrechen'],
	'action.create': ['Create', 'Créer', 'Crear', 'Erstellen'],
	'action.retry': ['Retry', 'Réessayer', 'Reintentar', 'Erneut versuchen'],
	'action.save': ['Save', 'Enregistrer', 'Guardar', 'Speichern'],
	'action.undo': ['Undo', 'Annuler', 'Deshacer', 'Rückgängig'],
	'decks.title': ['Presentations', 'Présentations', 'Presentaciones', 'Präsentationen'],
	'decks.subtitle': [
		'Each presentation is one text file.',
		'Chaque présentation est un seul fichier texte.',
		'Cada presentación es un solo archivo de texto.',
		'Jede Präsentation ist eine einzige Textdatei.'
	],
	'decks.new': ['New presentation', 'Nouvelle présentation', 'Nueva presentación', 'Neue Präsentation'],
	'decks.empty': ['No presentation yet', 'Aucune présentation', 'Ninguna presentación', 'Noch keine Präsentation'],
	'decks.empty.hint': [
		'Create one, write your slides as text and present them from here.',
		'Créez-en une, écrivez vos diapositives en texte et présentez-les d’ici.',
		'Cree una, escriba sus diapositivas como texto y preséntelas desde aquí.',
		'Erstellen Sie eine, schreiben Sie Ihre Folien als Text und präsentieren Sie von hier aus.'
	],
	'decks.failed': [
		'The presentations could not be loaded.',
		'Les présentations n’ont pas pu être chargées.',
		'No se pudieron cargar las presentaciones.',
		'Die Präsentationen konnten nicht geladen werden.'
	],
	'deck.title': ['Title', 'Titre', 'Título', 'Titel'],
	'deck.untitled': ['Untitled', 'Sans titre', 'Sin título', 'Ohne Titel'],
	'deck.rename': ['Rename', 'Renommer', 'Renombrar', 'Umbenennen'],
	'deck.trash': ['Move to trash', 'Mettre à la corbeille', 'Mover a la papelera', 'In den Papierkorb'],
	'deck.trashed': [
		'“{title}” moved to the trash',
		'« {title} » mise à la corbeille',
		'«{title}» movida a la papelera',
		'„{title}“ in den Papierkorb verschoben'
	],
	'deck.actions': ['Actions for {title}', 'Actions pour {title}', 'Acciones para {title}', 'Aktionen für {title}'],
	'deck.missing': [
		'Presentation not found',
		'Présentation introuvable',
		'Presentación no encontrada',
		'Präsentation nicht gefunden'
	],
	'deck.missing.hint': [
		'It was deleted, or you do not have access to it.',
		'Elle a été supprimée, ou vous n’y avez pas accès.',
		'Fue eliminada, o no tiene acceso a ella.',
		'Sie wurde gelöscht, oder Sie haben keinen Zugriff darauf.'
	],
	'deck.back': [
		'Back to presentations',
		'Retour aux présentations',
		'Volver a las presentaciones',
		'Zurück zu den Präsentationen'
	],
	'trash.title': ['Trash', 'Corbeille', 'Papelera', 'Papierkorb'],
	'trash.subtitle': [
		'Trashed presentations. Restore one, or delete it for good.',
		'Les présentations mises à la corbeille. Restaurez-en une, ou supprimez-la définitivement.',
		'Las presentaciones en la papelera. Restaure una, o elimínela definitivamente.',
		'Präsentationen im Papierkorb. Stellen Sie eine wieder her oder löschen Sie sie endgültig.'
	],
	'trash.empty': ['The trash is empty', 'La corbeille est vide', 'La papelera está vacía', 'Der Papierkorb ist leer'],
	'trash.restore': ['Restore', 'Restaurer', 'Restaurar', 'Wiederherstellen'],
	'trash.delete': ['Delete for good', 'Supprimer définitivement', 'Eliminar definitivamente', 'Endgültig löschen'],
	'trash.confirm': [
		'Delete “{title}” for good?',
		'Supprimer définitivement « {title} » ?',
		'¿Eliminar definitivamente «{title}»?',
		'„{title}“ endgültig löschen?'
	],
	'trash.confirm.hint': [
		'The presentation, its history and its files are deleted. This cannot be undone.',
		'La présentation, son historique et ses fichiers sont supprimés. Cette action est irréversible.',
		'La presentación, su historial y sus archivos se eliminan. Esta acción no se puede deshacer.',
		'Die Präsentation, ihr Verlauf und ihre Dateien werden gelöscht. Das lässt sich nicht rückgängig machen.'
	],
	'settings.title': ['Settings', 'Réglages', 'Ajustes', 'Einstellungen'],
	'settings.profile': ['Profile', 'Profil', 'Perfil', 'Profil'],
	'settings.preferences': ['Preferences', 'Préférences', 'Preferencias', 'Einstellungen'],
	'error.title': ['Error', 'Erreur', 'Error', 'Fehler'],
	'error.generic': [
		'Something went wrong',
		'Une erreur est survenue',
		'Se produjo un error',
		'Ein Fehler ist aufgetreten'
	],
	'error.hint': [
		'Reload the page. If the error comes back, try again later.',
		'Rechargez la page. Si l’erreur revient, réessayez plus tard.',
		'Recargue la página. Si el error vuelve, inténtelo más tarde.',
		'Laden Sie die Seite neu. Tritt der Fehler erneut auf, versuchen Sie es später.'
	],
	'error.missing': ['Page not found', 'Page introuvable', 'Página no encontrada', 'Seite nicht gefunden'],
	'error.missing.hint': [
		'This address leads to no page of Diapo.',
		'Cette adresse ne mène à aucune page de Diapo.',
		'Esta dirección no lleva a ninguna página de Diapo.',
		'Diese Adresse führt zu keiner Seite von Diapo.'
	]
} satisfies Record<string, Row>;
