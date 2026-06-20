// Dueo es selfhosted y se embebe en el binario: lo servimos como SPA (sin SSR).
// Así la sesión por cookie es 100% cliente y simple.
export const ssr = false;
export const prerender = false;
