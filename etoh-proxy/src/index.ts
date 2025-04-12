import { generateDocumentation } from '../docs/generator';
import { fetchResponse, handleApiRequest } from './wrappers';

import { getTowerData, getAllTowerData, compareBadges } from './apis/badges';
import { getNameFromId, getIdFromName } from './apis/users';

async function getRequestDetails(request: Request) {
	let url = new URL(request.url);
	let details = url.pathname.split('/');
	let route = details[1];
	details.shift();
	details.shift();

	return { route, details };
}

function processDetails(details: string[], bindings: Bindings) {
	let response: BindingsResponse = {
		error: {}
	};

	Object.entries(bindings).forEach(([key, value]) => {
		if (value.position > details.length && value.required) {
			response.error.position = `${key} requested a pos of ${value.position} which does not exist in. Defaulting to empty.`;
			switch (value.type) {
				default:
				case 'string': response[key] = ''; break;
				case 'number': response[key] = 0; break;
			}
		}

		let info = details[value.position];
		switch (value.type) {
			default:
			case 'string':
				response[key] = info;
				break;
			case 'number':
				response[key] = Number(info);
				if (isNaN(response[key])) {
					response.error[key] = `Failed to parse ${info} into a number. Please check the passed parameters.`;
				}
				break;
		}
	})

	return response;
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
				let { user_id: userId, username, option: userOption, error } = processDetails(details, {
					user_id: {
						position: 0,
						type: 'number',
						required: false,
					},
					username: {
						position: 0,
						type: 'string',
						required: false,
					},
					option: {
						position: 1,
						type: 'string',
						required: false
					}
				});

				if (Object.keys(error).length > 0) {
					return Response.json(error);
				}

				switch (userOption) {
					case 'name':
						console.log(`Getting user name from ${userId}`);
						return handleApiRequest(getNameFromId(userId));

					case '':
					default:
						console.log(`Getting user id from ${username}`);
						return handleApiRequest(getIdFromName(username));
				}
			case 'towers':
				let { user_id: towerId, option: towerOption, badge_1, badge_2 } = processDetails(details, {
					user_id: {
						type: 'number',
						position: 0,
						required: true,
					},
					option: {
						type: 'string',
						position: 1,
						required: true,
					},
					badge_1: {
						type: 'number',
						position: 2,
						required: false,
					},
					badge_2: {
						type: 'number',
						position: 3,
						required: false,
					}
				});


				console.log(`Getting badge data for ${towerId} (${{ option: towerOption, badge_1, badge_2 }})`);

				switch (towerOption) {
					case 'compare':
						return handleApiRequest(compareBadges(towerId, badge_1, badge_2));
					case 'all':
						let badges: { badgeids: number[] } = await request.json();
						return handleApiRequest(getAllTowerData(towerId, badges.badgeids));
					case 'badge':
					default:
						return handleApiRequest(getTowerData(towerId, badge_1));
				}

			default:
			case '':
				return fetchResponse(generateDocumentation(), {
					headers: {
						'Content-Type': 'text/html'
					}
				});
		}
	},
} satisfies ExportedHandler<Env>;
