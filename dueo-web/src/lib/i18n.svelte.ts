// Minimal i18n (no library), Svelte 5 runes. Flat dictionary and a reactive
// `t(key, params?)`: reading `lang` inside t makes any component calling
// i18n.t(...) in its markup re-render on language change. Persisted in localStorage.
//
// ADDING A LANGUAGE: 1) add it to LANGS; 2) add its key (e.g. `fr`) to the DICT
// entries. No need to translate everything at once: anything missing falls back to
// the default language (DEFAULT_LANG), and failing that, to the key itself.

// Available languages (order is the selector's order). To add one, append here.
export const LANGS = [
	{ code: 'es', label: 'Español' },
	{ code: 'en', label: 'English' }
] as const;

export type Lang = string; // a LANGS code (string so we're not locked to 2)

const DEFAULT_LANG: Lang = 'es';
const isLang = (v: string): boolean => LANGS.some((l) => l.code === v);

// Each entry maps a language code → text. It need not include every language:
// missing ones use the fallback.
type Entry = Record<string, string>;

const DICT: Record<string, Entry> = {
	// --- Navigation (Sidebar) ---
	'nav.dashboard': { es: 'Dashboard', en: 'Dashboard' },
	'nav.insights': { es: 'Insights', en: 'Insights' },
	'nav.categories': { es: 'Categorías', en: 'Categories' },
	'nav.settings': { es: 'Ajustes', en: 'Settings' },
	'nav.aria': { es: 'Navegación', en: 'Navigation' },

	// --- Error page ---
	'err.notFoundTitle': { es: 'Página no encontrada', en: 'Page not found' },
	'err.notFoundText': {
		es: 'La página que buscas no existe o se movió.',
		en: "The page you're looking for doesn't exist or was moved."
	},
	'err.genericTitle': { es: 'Algo salió mal', en: 'Something went wrong' },
	'err.genericText': {
		es: 'Ocurrió un error inesperado. Inténtalo de nuevo.',
		en: 'An unexpected error occurred. Please try again.'
	},
	'err.home': { es: 'Volver al inicio', en: 'Back to home' },

	// --- Topbar ---
	'topbar.home': { es: 'Dueo · inicio', en: 'Dueo · home' },
	'topbar.logout': { es: 'Cerrar sesión', en: 'Log out' },
	'topbar.theme': { es: 'Cambiar tema', en: 'Toggle theme' },
	'topbar.lang': { es: 'Cambiar idioma', en: 'Change language' },
	'topbar.menu': { es: 'Menú', en: 'Menu' },

	// --- Document title (layout) ---
	'doc.home': { es: 'Inicio', en: 'Home' },
	'doc.login': { es: 'Acceder', en: 'Sign in' },
	'doc.categories': { es: 'Categorías', en: 'Categories' },
	'doc.insights': { es: 'Insights', en: 'Insights' },
	'doc.settings': { es: 'Ajustes', en: 'Settings' },

	// --- Login ---
	'login.subtitleLogin': {
		es: 'Inicia sesión para ver tus suscripciones',
		en: 'Sign in to see your subscriptions'
	},
	'login.subtitleRegister': {
		es: 'Crea tu cuenta para empezar',
		en: 'Create your account to get started'
	},
	'login.setup': {
		es: 'Primer arranque: crea la cuenta de administrador.',
		en: 'First run: create the administrator account.'
	},
	'login.user': { es: 'Usuario', en: 'Username' },
	'login.password': { es: 'Contraseña', en: 'Password' },
	'login.passwordMin': { es: 'Contraseña (mín. 8)', en: 'Password (min. 8)' },
	'login.showPass': { es: 'Mostrar contraseña', en: 'Show password' },
	'login.hidePass': { es: 'Ocultar contraseña', en: 'Hide password' },
	'login.capsOn': { es: 'Bloq Mayús activado', en: 'Caps Lock is on' },
	'login.weak': { es: 'Débil', en: 'Weak' },
	'login.medium': { es: 'Media', en: 'Medium' },
	'login.strong': { es: 'Fuerte', en: 'Strong' },
	'login.signIn': { es: 'Entrar', en: 'Sign in' },
	'login.createAccount': { es: 'Crear cuenta', en: 'Create account' },
	'login.toRegister': { es: '¿No tienes cuenta? Crear una', en: 'No account? Create one' },
	'login.toLogin': { es: '¿Ya tienes cuenta? Entrar', en: 'Already have an account? Sign in' },
	'login.errMin': {
		es: 'La contraseña debe tener al menos 8 caracteres',
		en: 'Password must be at least 8 characters'
	},
	'login.errExists': { es: 'Ese usuario ya existe', en: 'That username already exists' },
	'login.errClosed': {
		es: 'El registro está cerrado. Pide a un administrador que cree tu cuenta.',
		en: 'Registration is closed. Ask an administrator to create your account.'
	},
	'login.errCreate': { es: 'No se pudo crear la cuenta', en: 'Could not create the account' },
	'login.errNoEntry': {
		es: 'Cuenta creada, pero no se pudo entrar',
		en: 'Account created, but could not sign in'
	},
	'login.errInvalid': { es: 'Credenciales inválidas', en: 'Invalid credentials' },
	'login.errTooMany': {
		es: 'Demasiados intentos. Inténtalo de nuevo más tarde.',
		en: 'Too many attempts. Try again later.'
	},
	'login.errConn': {
		es: 'No se pudo conectar con el servidor',
		en: 'Could not connect to the server'
	},

	// --- Common ---
	'common.save': { es: 'Guardar', en: 'Save' },
	'common.saving': { es: 'Guardando…', en: 'Saving…' },
	'common.cancel': { es: 'Cancelar', en: 'Cancel' },
	'common.create': { es: 'Crear', en: 'Create' },
	'common.add': { es: 'Añadir', en: 'Add' },
	'common.close': { es: 'Cerrar', en: 'Close' },
	'common.clear': { es: 'Limpiar', en: 'Clear' },
	'common.actionError': {
		es: 'No se pudo completar la acción. Inténtalo de nuevo.',
		en: "Couldn't complete the action. Try again."
	},
	'ring.daysLeft': { es: 'días restantes', en: 'days left' },
	'common.edit': { es: 'Editar', en: 'Edit' },
	'common.delete': { es: 'Borrar', en: 'Delete' },
	'common.confirmDelete': { es: 'Sí, borrar', en: 'Yes, delete' },
	'common.connError': { es: 'Error de conexión', en: 'Connection error' },

	// --- Dashboard ---
	'dash.title': { es: 'Suscripciones', en: 'Subscriptions' },
	'dash.serviceOne': { es: 'servicio', en: 'service' },
	'dash.serviceMany': { es: 'servicios', en: 'services' },
	'dash.add': { es: 'Agregar', en: 'Add' },
	'dash.emptyTitle': {
		es: 'Aún no tienes suscripciones',
		en: "You don't have any subscriptions yet"
	},
	'dash.emptyText': {
		es: 'Agrega tu primer servicio y empieza a ver cuánto le queda.',
		en: 'Add your first service and start seeing how much time it has left.'
	},
	'dash.addFirst': { es: 'Agregar la primera', en: 'Add the first one' },
	'dash.monthly': { es: 'Gasto mensual', en: 'Monthly spend' },
	'dash.annual': { es: 'Gasto anual', en: 'Annual spend' },
	'dash.noConversion': { es: 'sin conversión', en: 'no conversion' },
	'dash.nextDue': { es: 'Próximo vencimiento', en: 'Next due' },
	'dash.inDays': { es: 'en {n} días', en: 'in {n} days' },
	'dash.showing': { es: 'Mostrando:', en: 'Showing:' },
	'dash.showAll': { es: 'Ver todas', en: 'Show all' },
	'dash.removeFilter': { es: 'Quitar filtro', en: 'Remove filter' },
	'dash.searchPlaceholder': { es: 'Buscar suscripción…', en: 'Search subscription…' },
	'dash.allCategories': { es: 'Todas las categorías', en: 'All categories' },
	'dash.noCategory': { es: 'Sin categoría', en: 'No category' },
	'dash.categoryAria': { es: 'Categoría', en: 'Category' },
	'dash.sortAria': { es: 'Ordenar', en: 'Sort' },
	'dash.sortDue': { es: 'Próximo vencimiento', en: 'Next due' },
	'dash.sortAmount': { es: 'Mayor gasto', en: 'Highest spend' },
	'dash.sortName': { es: 'Nombre A–Z', en: 'Name A–Z' },
	'dash.noResults': {
		es: 'Ninguna suscripción coincide con esos filtros.',
		en: 'No subscription matches those filters.'
	},
	'dash.statusAll': { es: 'Todas', en: 'All' },
	'dash.statusActive': { es: 'Activas', en: 'Active' },
	'dash.statusExpired': { es: 'Vencidas', en: 'Overdue' },
	'dash.statusPaused': { es: 'Pausadas', en: 'Paused' },

	// --- Subscription row (statuses + actions) ---
	'row.active': { es: 'Activa', en: 'Active' },
	'row.paused': { es: 'Pausada', en: 'Paused' },
	'row.expired': { es: 'Vencida', en: 'Overdue' },
	'row.actions': { es: 'Acciones', en: 'Actions' },
	'row.closeMenu': { es: 'Cerrar menú', en: 'Close menu' },
	'row.renew': { es: 'Renovar', en: 'Renew' },

	// --- Time / cycle / reminder labels ---
	'fmt.overdue': { es: 'vencida', en: 'overdue' },
	'fmt.dueToday': { es: 'vence hoy', en: 'due today' },
	'fmt.dueTomorrow': { es: 'vence mañana', en: 'due tomorrow' },
	'fmt.dueInDays': { es: 'vence en {n} días', en: 'due in {n} days' },
	'fmt.dueInMonth': { es: 'vence en ~1 mes', en: 'due in ~1 month' },
	'fmt.dueInMonths': { es: 'vence en ~{n} meses', en: 'due in ~{n} months' },
	'cycle.monthly': { es: 'mensual', en: 'monthly' },
	'cycle.yearly': { es: 'anual', en: 'yearly' },
	'cycle.once': { es: 'único', en: 'one-time' },
	'cycle.days': { es: '{n} días', en: '{n} days' },
	'rem.sameDay': { es: 'el mismo día', en: 'same day' },
	'rem.oneDay': { es: '1 día antes', en: '1 day before' },
	'rem.days': { es: '{n} días antes', en: '{n} days before' },
	'ago.now': { es: 'ahora', en: 'now' },
	'ago.min': { es: 'hace {n} min', en: '{n} min ago' },
	'ago.hour': { es: 'hace {n} h', en: '{n}h ago' },
	'ago.day': { es: 'hace {n} d', en: '{n}d ago' },

	// --- Subscription modal ---
	'modal.newSub': { es: 'Nueva suscripción', en: 'New subscription' },
	'modal.name': { es: 'Nombre', en: 'Name' },
	'modal.namePlaceholder': { es: 'Netflix, Dominio, Hosting…', en: 'Netflix, Domain, Hosting…' },
	'modal.iconColor': { es: 'Icono y color', en: 'Icon and color' },
	'modal.auto': { es: 'Auto', en: 'Auto' },
	'modal.iconSearch': {
		es: 'Buscar marca o icono (Disney, gimnasio…)',
		en: 'Search brand or icon (Disney, gym…)'
	},
	'modal.loadingBrands': { es: 'Cargando marcas…', en: 'Loading brands…' },
	'modal.color': { es: 'Color', en: 'Color' },
	'modal.customColor': { es: 'Color personalizado', en: 'Custom color' },
	'modal.amount': { es: 'Monto', en: 'Amount' },
	'modal.currency': { es: 'Moneda', en: 'Currency' },
	'modal.cycle': { es: 'Ciclo', en: 'Cycle' },
	'modal.cycleMonthly': { es: 'Mensual', en: 'Monthly' },
	'modal.cycleYearly': { es: 'Anual', en: 'Yearly' },
	'modal.cycleCustom': { es: 'Personalizado', en: 'Custom' },
	'modal.cycleOnce': { es: 'Único', en: 'One-time' },
	'modal.everyDays': { es: 'Cada (días)', en: 'Every (days)' },
	'modal.payment': { es: 'Pago', en: 'Payment' },
	'modal.paymentManual': { es: 'Manual', en: 'Manual' },
	'modal.paymentAuto': { es: 'Domiciliado', en: 'Auto-pay' },
	'modal.start': { es: 'Inicio', en: 'Start' },
	'modal.due': { es: 'Vencimiento', en: 'Due date' },
	'modal.category': { es: 'Categoría', en: 'Category' },
	'modal.noCategory': { es: 'Sin categoría', en: 'No category' },
	'modal.remTitle': { es: 'Recordatorios de este servicio', en: 'Reminders for this service' },
	'modal.remHint': {
		es: 'Si añades alguno, sustituye a los avisos globales para este servicio.',
		en: 'If you add any, they replace the global reminders for this service.'
	},
	'modal.remUseGlobal': { es: 'Usa los avisos globales.', en: 'Uses the global reminders.' },
	'modal.daysBefore': { es: 'días antes', en: 'days before' },
	'modal.errName': { es: 'Pon un nombre', en: 'Enter a name' },
	'modal.errAmount': { es: 'Monto inválido', en: 'Invalid amount' },
	'modal.errDates': { es: 'Faltan las fechas', en: 'Dates are missing' },
	'modal.errSave': { es: 'No se pudo guardar', en: 'Could not save' },
	'modal.errCreate': {
		es: 'No se pudo crear la suscripción',
		en: 'Could not create the subscription'
	},

	// --- Notifications ---
	'notif.title': { es: 'Notificaciones', en: 'Notifications' },
	'notif.markAll': { es: 'Marcar todo', en: 'Mark all' },
	'notif.empty': { es: 'Sin notificaciones todavía.', en: 'No notifications yet.' },

	// --- Horizon ---
	'hz.title': { es: 'Horizonte de vencimientos', en: 'Due-date horizon' },
	'hz.hint': {
		es: 'pellizca o Ctrl+rueda = zoom · arrastra = mover',
		en: 'pinch or Ctrl+wheel = zoom · drag = pan'
	},
	'hz.viewInList': { es: 'Ver en la lista', en: 'View in list' },
	'hz.now': { es: 'HOY', en: 'NOW' },
	'hz.all': { es: 'Todo', en: 'All' },

	// --- Categories ---
	'cat.title': { es: 'Categorías', en: 'Categories' },
	'cat.new': { es: 'Nueva', en: 'New' },
	'cat.hint': {
		es: 'Agrupa tus suscripciones por color para leerlas de un vistazo.',
		en: 'Group your subscriptions by color to read them at a glance.'
	},
	'cat.nameFull': { es: 'Nombre de la categoría', en: 'Category name' },
	'cat.name': { es: 'Nombre', en: 'Name' },
	'cat.confirmDeleteAria': { es: 'Confirmar borrado', en: 'Confirm delete' },
	'cat.empty': {
		es: 'Aún no tienes categorías. Crea la primera para organizar tus pagos.',
		en: "You don't have any categories yet. Create the first one to organize your payments."
	},

	// --- Insights ---
	'ins.title': { es: 'Insights', en: 'Insights' },
	'ins.empty': {
		es: 'Aún no hay datos. Agrega suscripciones para ver tus insights.',
		en: 'No data yet. Add subscriptions to see your insights.'
	},
	'ins.subscriptions': { es: 'Suscripciones', en: 'Subscriptions' },
	'ins.activeExpired': { es: '{a} activas · {e} vencidas', en: '{a} active · {e} overdue' },
	'ins.monthly': { es: 'Gasto mensual', en: 'Monthly spend' },
	'ins.annual': { es: 'Gasto anual', en: 'Annual spend' },
	'ins.dueSoon': { es: 'Vencen pronto', en: 'Due soon' },
	'ins.next7': { es: 'en los próximos 7 días', en: 'in the next 7 days' },
	'ins.byCategory': { es: 'Gasto por categoría', en: 'Spend by category' },
	'ins.only': { es: 'solo {cur}', en: '{cur} only' },
	'ins.perMonth': { es: 'al mes', en: 'per month' },
	'ins.top': { es: 'Top por gasto', en: 'Top by spend' },
	'ins.projected': {
		es: 'Gasto proyectado · próximos 6 meses',
		en: 'Projected spend · next 6 months'
	},
	'ins.noCategory': { es: 'Sin categoría', en: 'No category' },

	// --- Settings ---
	'set.title': { es: 'Ajustes', en: 'Settings' },
	'set.prefTitle': { es: 'Preferencias', en: 'Preferences' },
	'set.prefDesc': {
		es: 'Tu zona horaria y la hora local a la que te enviamos el aviso diario.',
		en: 'Your time zone and the local time we send your daily reminder.'
	},
	'set.timezone': { es: 'Zona horaria', en: 'Time zone' },
	'set.sendHour': { es: 'Hora de aviso', en: 'Reminder time' },
	'set.mainCurrency': { es: 'Moneda principal', en: 'Main currency' },
	'set.savedOk': { es: 'Guardado ✓', en: 'Saved ✓' },
	'set.saveErr': { es: 'No se pudo guardar', en: 'Could not save' },
	'set.remTitle': { es: 'Recordatorios', en: 'Reminders' },
	'set.remDesc': {
		es: 'Cuántos días antes del vencimiento te avisamos. Aplica a los servicios que no tengan reglas propias.',
		en: 'How many days before the due date we notify you. Applies to services without their own rules.'
	},
	'set.remEmpty': {
		es: 'Sin anticipaciones. Añade al menos una para recibir avisos.',
		en: 'No reminders set. Add at least one to receive notices.'
	},
	'set.remPlaceholder': { es: 'días antes (p. ej. 7)', en: 'days before (e.g. 7)' },
	'set.remErrInvalid': { es: 'Pon un número de días válido', en: 'Enter a valid number of days' },
	'set.remErrExists': { es: 'Ya existe esa anticipación', en: 'That reminder already exists' },
	'set.remErrAdd': { es: 'No se pudo añadir', en: 'Could not add' },
	'set.tgTitle': { es: 'Telegram', en: 'Telegram' },
	'set.tgDesc': {
		es: 'Recibe los recordatorios en un chat o grupo de Telegram.',
		en: 'Receive reminders in a Telegram chat or group.'
	},
	'set.tgNoTokenPre': {
		es: 'El servidor no tiene token de Telegram configurado (',
		en: 'The server has no Telegram token configured ('
	},
	'set.tgChatId': { es: 'Chat ID', en: 'Chat ID' },
	'set.tgChatHint': {
		es: 'Para grupos suele ser un número negativo. El bot debe estar dentro del chat.',
		en: "For groups it's usually a negative number. The bot must be in the chat."
	},
	'set.tgEnable': { es: 'Activar envíos por Telegram', en: 'Enable Telegram delivery' },
	'set.tgTest': { es: 'Enviar prueba', en: 'Send test' },
	'set.tgTesting': { es: 'Enviando…', en: 'Sending…' },
	'set.tgTestOk': { es: 'Mensaje de prueba enviado ✓', en: 'Test message sent ✓' },
	'set.tgTestErr': { es: 'No se pudo enviar', en: 'Could not send' },
	'set.emTitle': { es: 'Correo', en: 'Email' },
	'set.emDesc': {
		es: 'Recibe los recordatorios por correo electrónico.',
		en: 'Receive reminders by email.'
	},
	'set.emNoSmtpPre': {
		es: 'El servidor no tiene SMTP configurado (',
		en: 'The server has no SMTP configured ('
	},
	'set.emAddress': { es: 'Tu correo', en: 'Your email' },
	'set.emHint': {
		es: 'A esta dirección llegarán los avisos. El servidor debe tener un SMTP configurado.',
		en: 'Reminders will be sent to this address. The server must have SMTP configured.'
	},
	'set.emEnable': { es: 'Activar envíos por correo', en: 'Enable email delivery' },
	'set.emTestOk': { es: 'Correo de prueba enviado ✓', en: 'Test email sent ✓' },
	'set.emTestErr': { es: 'No se pudo enviar', en: 'Could not send' },
	'set.dataTitle': { es: 'Datos', en: 'Data' },
	'set.dataDesc': {
		es: 'Exporta una copia de tus categorías, servicios y recordatorios, o impórtala en esta cuenta.',
		en: 'Export a copy of your categories, services and reminders, or import it into this account.'
	},
	'set.export': { es: 'Exportar', en: 'Export' },
	'set.import': { es: 'Importar', en: 'Import' },
	'set.importing': { es: 'Importando…', en: 'Importing…' },
	'set.dataHint': {
		es: 'Importar añade los datos del archivo a tu cuenta (no reemplaza los existentes).',
		en: "Importing adds the file's data to your account (it does not replace existing data)."
	},
	'set.exportOk': { es: 'Datos exportados', en: 'Data exported' },
	'set.exportErr': { es: 'No se pudo exportar', en: 'Could not export' },
	'set.importOk': {
		es: 'Importado: {c} categorías, {s} servicios, {r} recordatorios',
		en: 'Imported: {c} categories, {s} services, {r} reminders'
	},
	'set.importErr': { es: 'No se pudo importar', en: 'Could not import' },
	'set.importInvalid': {
		es: 'El archivo no es un backup válido',
		en: 'The file is not a valid backup'
	},
	'set.usersTitle': { es: 'Usuarios', en: 'Users' },
	'set.usersDesc': {
		es: 'Cuentas de esta instancia. Borrar una elimina también todos sus datos.',
		en: 'Accounts on this instance. Deleting one also removes all its data.'
	},
	'set.you': { es: 'tú', en: 'you' },
	'set.deleteUser': { es: 'Borrar usuario', en: 'Delete user' },
	'set.userPlaceholder': { es: 'usuario', en: 'username' },
	'set.passPlaceholder': { es: 'contraseña', en: 'password' },
	'set.roleMember': { es: 'Miembro', en: 'Member' },
	'set.roleAdmin': { es: 'Administrador', en: 'Administrator' },
	'set.creating': { es: 'Creando…', en: 'Creating…' },
	'set.usersErrName': { es: 'Pon un nombre de usuario', en: 'Enter a username' },
	'set.usersErrPass': {
		es: 'La contraseña debe tener al menos 8 caracteres',
		en: 'Password must be at least 8 characters'
	},
	'set.usersErrCreate': { es: 'No se pudo crear', en: "Couldn't create user" },
	'set.usersErrDelete': { es: 'No se pudo borrar', en: "Couldn't delete user" },
	'set.secTitle': { es: 'Seguridad', en: 'Security' },
	'set.secDesc': {
		es: 'Cambia tu contraseña o cierra la sesión en todos los dispositivos.',
		en: 'Change your password or sign out on all devices.'
	},
	'set.currentPass': { es: 'Contraseña actual', en: 'Current password' },
	'set.newPass': { es: 'Nueva contraseña', en: 'New password' },
	'set.repeatPass': { es: 'Repite la nueva', en: 'Repeat the new one' },
	'set.changePass': { es: 'Cambiar contraseña', en: 'Change password' },
	'set.pwErrMin': {
		es: 'La nueva debe tener al menos 8 caracteres',
		en: 'The new one must be at least 8 characters'
	},
	'set.pwErrMatch': { es: 'Las contraseñas no coinciden', en: "Passwords don't match" },
	'set.pwOk': { es: 'Contraseña actualizada', en: 'Password updated' },
	'set.pwErr': { es: 'No se pudo cambiar', en: 'Could not change' },
	'set.closeAll': { es: 'Cerrar todas las sesiones', en: 'Sign out everywhere' },
	'set.closing': { es: 'Cerrando…', en: 'Signing out…' },
	'set.closeAllHint': {
		es: 'Cierra la sesión también en este dispositivo; tendrás que volver a entrar.',
		en: "Signs you out on this device too; you'll have to sign in again."
	},
	'set.aboutTitle': { es: 'Acerca de', en: 'About' },
	'set.aboutDesc': {
		es: 'Dueo es un panel de suscripciones de código abierto, autoalojado.',
		en: 'Dueo is an open-source, self-hosted subscriptions panel.'
	},
	'set.version': { es: 'Versión', en: 'Version' },
	'set.backend': { es: 'Backend', en: 'Backend' },
	'set.online': { es: 'En línea', en: 'Online' },
	'set.offline': { es: 'Sin conexión', en: 'Offline' },
	'set.project': { es: 'Proyecto', en: 'Project' },
	'set.repo': { es: 'Repositorio', en: 'Repository' },
	'set.update': { es: 'Actualización', en: 'Update' },
	'set.updateLatest': { es: 'Estás en la última versión', en: "You're on the latest version" },
	'set.updateAvail': {
		es: 'Disponible {version}',
		en: '{version} available'
	},
	'set.updateView': { es: 'Ver release', en: 'View release' },
	'set.updateHow': { es: 'Cómo actualizar', en: 'How to update' },
	'set.updateCmd': {
		es: 'En el servidor, ejecuta:',
		en: 'On the server, run:'
	}
};

function createI18n() {
	let lang = $state<Lang>(DEFAULT_LANG);

	return {
		get lang() {
			return lang;
		},
		init() {
			let l: Lang = DEFAULT_LANG;
			try {
				const saved = localStorage.getItem('lang');
				if (saved && isLang(saved)) l = saved;
				else {
					// from the browser: match the prefix (es-MX → es) to an available language.
					const nav = navigator.language?.toLowerCase().split('-')[0] ?? '';
					if (isLang(nav)) l = nav;
				}
			} catch {
				/* no storage: keep the default language */
			}
			lang = l;
			document.documentElement.lang = l;
		},
		set(l: Lang) {
			if (!isLang(l)) return;
			lang = l;
			try {
				localStorage.setItem('lang', l);
			} catch {
				/* ignore */
			}
			document.documentElement.lang = l;
		},
		// Translate a key; interpolate {param} if provided. Fallback: current language →
		// default language → the key itself (so an incomplete language breaks nothing).
		t(key: string, params?: Record<string, string | number>): string {
			const entry = DICT[key];
			let s = entry ? (entry[lang] ?? entry[DEFAULT_LANG] ?? key) : key; // read `lang` → reactive
			if (params) {
				for (const [k, v] of Object.entries(params)) s = s.replaceAll(`{${k}}`, String(v));
			}
			return s;
		}
	};
}

export const i18n = createI18n();

// Locale for Intl/toLocaleDateString (numbers, currencies, month names).
export function locale(): Lang {
	return i18n.lang;
}

// ---- Label helpers (use i18n.t → reactive to language) ----------------------

export function daysLabel(days: number): string {
	if (days < 0) return i18n.t('fmt.overdue');
	if (days === 0) return i18n.t('fmt.dueToday');
	if (days === 1) return i18n.t('fmt.dueTomorrow');
	if (days < 30) return i18n.t('fmt.dueInDays', { n: days });
	const m = Math.round(days / 30);
	return m === 1 ? i18n.t('fmt.dueInMonth') : i18n.t('fmt.dueInMonths', { n: m });
}

export function cycleLabel(cycle: string, days: number | null): string {
	if (cycle === 'monthly') return i18n.t('cycle.monthly');
	if (cycle === 'yearly') return i18n.t('cycle.yearly');
	if (cycle === 'once') return i18n.t('cycle.once');
	return i18n.t('cycle.days', { n: days ?? 0 });
}

export function reminderLabel(n: number): string {
	if (n === 0) return i18n.t('rem.sameDay');
	if (n === 1) return i18n.t('rem.oneDay');
	return i18n.t('rem.days', { n });
}

export function statusLabel(status: string): string {
	if (status === 'paused') return i18n.t('row.paused');
	if (status === 'expired') return i18n.t('row.expired');
	return i18n.t('row.active');
}
