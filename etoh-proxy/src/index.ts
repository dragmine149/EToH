import { generateDocumentation } from '../docs/generator';
import { fetchResponse, handleApiRequest } from './wrappers';

import { getTowerData, getAllTowerData } from './apis/badges';
import { getNameFromId, getIdFromName } from './apis/users';

async function getRequestDetails(request: Request) {
	let url = new URL(request.url);
	let details = url.pathname.split('/');
	let route = details[1];
	details.shift();
	details.shift();

	return { route, details };
}

export default {
	async fetch(request: Request, env: Object, ctx: ExecutionContext): Promise<Response> {
		// If the request method is OPTIONS, return CORS headers.
		if (
			request.method === "OPTIONS" &&
			request.headers.has("Origin") &&
			request.headers.has("Access-Control-Request-Method")
		) {
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


		let { route, details } = await getRequestDetails(request);
		console.log({ route, details });

		switch (route) {
			case 'users':
				if (details[3] == 'name') {
					console.log(`Getting user name from ${details[2]}`);
					return handleApiRequest(getNameFromId(parseInt(details[2])));
				}

				console.log(`Getting user id from ${details[2]}`);
				return handleApiRequest(getIdFromName(details[2]));
			case 'towers':
				console.log(`Getting badge data for ${details[3]}`);

				if (details[3] === 'all') {
					// console.log(request);
					// console.log(await request.text());
					// console.log(await request.json());
					let badges: { badgeids: number[] } = await request.json();
					return handleApiRequest(getAllTowerData(parseInt(details[2]), badges.badgeids));
					// return getAllTowerData(parseInt(details[2]), badges.badgeids);
				}

				return handleApiRequest(getTowerData(parseInt(details[3]), parseInt(details[2])));

			case '':
				return fetchResponse(generateDocumentation(), {
					headers: {
						'Content-Type': 'text/html'
					}
				});

			default:
				return new Response('Not Found', { status: 404 });
		}
	},
} satisfies ExportedHandler<Env>;
