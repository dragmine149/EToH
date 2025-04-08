import { tryCatch } from './utils';

// Wrapper to set origin for CORS
export function fetchRequest(input: RequestInfo | URL, init?: RequestInit) {
	init = init || {};
	init.headers = {
		...init.headers,
		Origin: "",
	};

	return new Request(input, init);
}

// wrapper to add headers for CORS
export function fetchResponse(body: BodyInit | null, init?: ResponseInit) {
	init = init || {};

	let headers = new Headers(init.headers);
	headers.set("Access-Control-Allow-Origin", "*");
	headers.set("Access-Control-Allow-Credentials", "true");
	headers.set("Access-Control-Allow-Methods", "*");

	init.headers = headers;
	return new Response(body, init);
}

// wrapper to make sure the response is fine.
export function ResponseResponse(response: Response) {
	return fetchResponse(response.body, {
		status: response.status,
		statusText: response.statusText,
		headers: response.headers
	});
}

export async function handleApiRequest(
	operation: Promise<Response>,
	errorStatus: number = 400
): Promise<Response> {
	const response = await tryCatch(operation);

	if (response.error) {
		return fetchResponse(response.error.message, { status: errorStatus });
	}

	return ResponseResponse(response.data);
}
