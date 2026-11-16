import type { Row } from '../row';

/** The editor: source, preview, scene strip, assets, insert menu, save states and Bulle. */
export const editor = {
	'editor.source': ['Source', 'Source', 'Fuente', 'Quelltext'],
	'editor.views': ['Editor views', 'Vues de l’éditeur', 'Vistas del editor', 'Editoransichten'],
	'preview.title': ['Preview', 'Aperçu', 'Vista previa', 'Vorschau'],
	'save.clean': ['Saved', 'Enregistré', 'Guardado', 'Gespeichert'],
	'save.dirty': [
		'Unsaved changes',
		'Modifications non enregistrées',
		'Cambios sin guardar',
		'Ungespeicherte Änderungen'
	],
	'save.saving': ['Saving…', 'Enregistrement…', 'Guardando…', 'Speichern…'],
	'save.invalid': [
		'Not saved: fix the source',
		'Non enregistré : corrigez la source',
		'No guardado: corrija la fuente',
		'Nicht gespeichert: Quelltext korrigieren'
	],
	'save.conflict': ['Conflict', 'Conflit', 'Conflicto', 'Konflikt'],
	'save.failed': ['Save failed', 'Échec de l’enregistrement', 'Error al guardar', 'Speichern fehlgeschlagen'],
	'conflict.title': [
		'This presentation changed elsewhere',
		'Cette présentation a changé ailleurs',
		'Esta presentación cambió en otro lugar',
		'Diese Präsentation wurde anderswo geändert'
	],
	'conflict.body': [
		'Someone else, an agent or another tab saved a newer version while you had unsaved text.',
		'Une autre personne, un agent ou un autre onglet a enregistré une version plus récente alors que votre texte n’était pas enregistré.',
		'Otra persona, un agente u otra pestaña guardó una versión más reciente mientras usted tenía texto sin guardar.',
		'Eine andere Person, ein Agent oder ein anderer Tab hat eine neuere Version gespeichert, während Ihr Text ungespeichert war.'
	],
	'conflict.reload': ['Reload theirs', 'Recharger leur version', 'Cargar su versión', 'Deren Version laden'],
	'conflict.overwrite': [
		'Keep mine and overwrite',
		'Garder la mienne et écraser',
		'Conservar la mía y sobrescribir',
		'Meine behalten und überschreiben'
	],
	'errors.title': [
		'Problems in the source',
		'Problèmes dans la source',
		'Problemas en la fuente',
		'Probleme im Quelltext'
	],
	'errors.line': ['Line {line}:', 'Ligne {line} :', 'Línea {line}:', 'Zeile {line}:'],
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
	'strip.title': ['Scenes', 'Scènes', 'Escenas', 'Szenen'],
	'strip.scene': ['Scene {scene}', 'Scène {scene}', 'Escena {scene}', 'Szene {scene}'],
	'strip.count': [
		{ one: '{count} scene', other: '{count} scenes' },
		{ one: '{count} scène', other: '{count} scènes' },
		{ one: '{count} escena', other: '{count} escenas' },
		{ one: '{count} Szene', other: '{count} Szenen' }
	],
	'strip.steps': [
		{ one: '{count} step', other: '{count} steps' },
		{ one: '{count} étape', other: '{count} étapes' },
		{ one: '{count} paso', other: '{count} pasos' },
		{ one: '{count} Schritt', other: '{count} Schritte' }
	],
	'insert.label': ['Insert', 'Insérer', 'Insertar', 'Einfügen'],
	'insert.cover': ['Cover', 'Couverture', 'Portada', 'Titelfolie'],
	'insert.bullets': ['Bullets with steps', 'Liste à étapes', 'Lista por pasos', 'Liste mit Schritten'],
	'insert.split': ['Split with image', 'Deux colonnes avec image', 'Dos columnas con imagen', 'Zwei Spalten mit Bild'],
	'insert.raw': ['Animated title (raw)', 'Titre animé (raw)', 'Título animado (raw)', 'Animierter Titel (raw)'],
	'template.title': ['Title', 'Titre', 'Título', 'Titel'],
	'template.subtitle': [
		'A line that says what this is about.',
		'Une ligne qui dit de quoi il s’agit.',
		'Una línea que dice de qué se trata.',
		'Eine Zeile, die sagt, worum es geht.'
	],
	'template.note': ['Presenter note.', 'Note pour l’orateur.', 'Nota para el orador.', 'Notiz für den Vortrag.'],
	'template.first': ['First point', 'Premier point', 'Primer punto', 'Erster Punkt'],
	'template.second': ['Second point', 'Deuxième point', 'Segundo punto', 'Zweiter Punkt'],
	'template.third': ['Third point', 'Troisième point', 'Tercer punto', 'Dritter Punkt'],
	'assets.title': ['Files', 'Fichiers', 'Archivos', 'Dateien'],
	'assets.hint': [
		'Pictures and other files this presentation uses.',
		'Les images et autres fichiers que cette présentation utilise.',
		'Las imágenes y otros archivos que usa esta presentación.',
		'Bilder und andere Dateien, die diese Präsentation verwendet.'
	],
	'assets.drop': ['Drop files here', 'Déposez des fichiers ici', 'Suelte archivos aquí', 'Dateien hier ablegen'],
	'assets.browse': ['Browse', 'Parcourir', 'Examinar', 'Durchsuchen'],
	'assets.empty': ['No file yet.', 'Aucun fichier pour l’instant.', 'Ningún archivo por ahora.', 'Noch keine Datei.'],
	'assets.copy': [
		'Copy the markdown reference',
		'Copier la référence markdown',
		'Copiar la referencia markdown',
		'Markdown-Verweis kopieren'
	],
	'assets.delete': ['Delete {name}', 'Supprimer {name}', 'Eliminar {name}', '{name} löschen'],
	'assets.failed': [
		'The file could not be sent.',
		'Le fichier n’a pas pu être envoyé.',
		'No se pudo enviar el archivo.',
		'Die Datei konnte nicht gesendet werden.'
	],
	'bulle.ask': ['Ask {name}', 'Demander à {name}', 'Preguntar a {name}', '{name} fragen']
} satisfies Record<string, Row>;
