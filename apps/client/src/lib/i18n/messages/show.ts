import type { Row } from '../row';

/** Present mode and the presenter view. */
export const show = {
	'present.start': ['Present', 'Présenter', 'Presentar', 'Präsentieren'],
	'present.exit': [
		'Leave the presentation',
		'Quitter la présentation',
		'Salir de la presentación',
		'Präsentation verlassen'
	],
	'present.fullscreen': ['Full screen (F)', 'Plein écran (F)', 'Pantalla completa (F)', 'Vollbild (F)'],
	'present.progress': ['Progress', 'Progression', 'Progreso', 'Fortschritt'],
	'presenter.open': ['Presenter view', 'Vue de l’orateur', 'Vista del orador', 'Referentenansicht'],
	'presenter.title': ['Presenter view', 'Vue de l’orateur', 'Vista del orador', 'Referentenansicht'],
	'presenter.current': ['Current scene', 'Scène en cours', 'Escena actual', 'Aktuelle Szene'],
	'presenter.next': ['Next scene', 'Scène suivante', 'Escena siguiente', 'Nächste Szene'],
	'presenter.end': [
		'End of the presentation',
		'Fin de la présentation',
		'Fin de la presentación',
		'Ende der Präsentation'
	],
	'presenter.notes': ['Notes', 'Notes', 'Notas', 'Notizen'],
	'presenter.nonotes': [
		'No note for this scene.',
		'Aucune note pour cette scène.',
		'Ninguna nota para esta escena.',
		'Keine Notiz zu dieser Szene.'
	],
	'timer.start': ['Start', 'Démarrer', 'Iniciar', 'Starten'],
	'timer.pause': ['Pause', 'Pause', 'Pausa', 'Pause'],
	'timer.reset': ['Reset the timer', 'Remettre le chrono à zéro', 'Reiniciar el cronómetro', 'Stoppuhr zurücksetzen']
} satisfies Record<string, Row>;
