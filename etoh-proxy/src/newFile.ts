import { tryCatch, getIdFromName, fetchResponse, ResponseResponse } from ".";

export default {
	async fetch(request: Request, env: Object, ctx: ExecutionContext): Promise<Response> {
		// If the request method is OPTIONS, return CORS headers.
		if (request.method === "OPTIONS" &&
			request.headers.has("Origin") &&
			request.headers.has("Access-Control-Request-Method")) {
			const responseHeaders = {
				"Access-Control-Allow-Origin": request.headers.get("Origin") || "*",
				"Access-Control-Allow-Methods": "*", // Allow all methods
				"Access-Control-Allow-Headers": request.headers.get(
					"Access-Control-Request-Headers"
				) || "*",
				"Access-Control-Max-Age": "86400",
			};
			return new Response(null, { headers: responseHeaders });
		}

		console.log('---------------------------------------------');

		// decode url to what we want
		let url = new URL(request.url);
		console.log({ path: url.pathname });
		let details = url.pathname.split('/');
		console.log({ details });
		let route = details[1];
		let param = details[2];

		// and handle it.
		console.log({ route, param });
		switch (route) {
			case 'users':
				console.log(`Getting user id from ${param}`);
				let response = await tryCatch(getIdFromName(param));
				if (response.error) {
					return fetchResponse(response.error.message, { status: 400 });
				}
				return ResponseResponse(response.data);
			default:
				return new Response('Not Found', { status: 404 });
		}
	},
} satisfies ExportedHandler<Env>;
