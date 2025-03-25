// tryCatch by theo - t3dotgg
// Types for the result object with discriminated union
type Success<T> = {
	data: T;
	error: null;
};

type Failure<E> = {
	data: null;
	error: E;
};

type Result<T, E = Error> = Success<T> | Failure<E>;

// Main wrapper function
export async function tryCatch<T, E = Error>(
	promise: Promise<T>,
): Promise<Result<T, E>> {
	try {
		const data = await promise;
		return { data, error: null };
	} catch (error) {
		return { data: null, error: error as E };
	}
}

type RobloxUser = {
	requestedUsername: string;
	hasVerifiedBadge: boolean;
	id: number;
	name: string;
	displayName: string;
}

type RobloxUserResponse = {
	data: RobloxUser[];
}

type BadgeResponse = {
	badgeId: number;
	awardedDate: string;
}




/**
* Gets the id of an user
* @param name The roblox USERNAME (not display name) of the user to get
* @returns The ID of the user
*/
async function getIdFromName(name: string): Promise<Response> {
	// if we have no name, then return as there is no reason for us to do anything.
	if (!name) {
		return new Response('Not Found', { status: 404 });
	}

	// Test for username as per the api.
	let response = await tryCatch(fetch(fetchRequest('https://users.roblox.com/v1/usernames/users', {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({
			usernames: [name],
			excludeBannedUsers: true
		})
	})));

	// if we have an error during the fetch
	if (response.error) {
		return new Response(`Failed to fetch user data: ${response.error.message}`, { status: 500 });
	}


	// decode the id from the data
	let data = await tryCatch<RobloxUserResponse>(response.data.json());
	if (data.error) {
		return new Response(`Failed to parse user data: ${data.error.message}`, { status: 500 });
	}

	let rbx_data = data.data.data;

	// just make sure we have data
	if (rbx_data?.length > 0) {
		return Response.json({
			id: rbx_data[0].id
		})
	}

	return new Response(`User not found`, { status: 404 });
}

/**
* Gets the awarded date of a badge
* @param badge_id The id of the badge
* @param user_id The id of the user
* @returns The date/time the badge got awarded
*/
async function getTowerData(badge_id: number, user_id: number) {
	let url = `https://badges.roblox.com/v1/users/${user_id}/badges/${badge_id}/awarded-date`;

	let response = await tryCatch(fetch(fetchRequest(url)));
	if (response.error) {
		return new Response(`Failed to fetch badge data: ${response.error.message}`, { status: 500 });
	}

	let data = await tryCatch<BadgeResponse>(response.data.json());
	if (data.error) {
		return new Response(`Failed to parse badge data: ${data.error.message}`, { status: 500 });
	}

	let rbx_data = data.data;

	if (rbx_data?.awardedDate) {
		return new Response(rbx_data.awardedDate);
	}

	return new Response(`Badge not found`, { status: 404 });
}


async function getAllTowerData(user_id: number, badges: number[]) {
	let url = `https://badges.roblox.com/v1/users/${user_id}/badges/awarded-dates`;

	let response = await tryCatch(fetch(fetchRequest(url, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({
			badgeIds: badges
		})
	})));
	if (response.error) {
		return new Response(`Failed to fetch badge data: ${response.error.message}`, { status: 500 });
	}

	let data = await tryCatch<BadgeResponse[]>(response.data.json());
	if (data.error) {
		return new Response(`Failed to parse badge data: ${data.error.message}`, { status: 500 });
	}

	let rbx_data = data.data;

	if (rbx_data?.length > 0) {
		return Response.json(rbx_data);
	}

	return new Response(`Badge not found`, { status: 404 });
}





// Wrapper to set origin for CORS
function fetchRequest(input: RequestInfo | URL, init?: RequestInit) {
	init = init || {};
	init.headers = {
		...init.headers,
		Origin: "",
	};

	return new Request(input, init);
}

// wrapper to add headers for CORS
function fetchResponse(body: BodyInit | null, init?: ResponseInit) {
	init = init || {};

	let headers = new Headers(init.headers);
	headers.set("Access-Control-Allow-Origin", "*");
	headers.set("Access-Control-Allow-Credentials", "true");
	headers.set("Access-Control-Allow-Methods", "*");

	init.headers = headers;
	return new Response(body, init);
}

// wrapper to make sure the response is fine.
function ResponseResponse(response: Response) {
	return fetchResponse(response.body, {
		status: response.status,
		statusText: response.statusText,
		headers: response.headers
	});
}

async function handleApiRequest(
	operation: Promise<Response>,
	errorStatus: number = 400
): Promise<Response> {
	const response = await tryCatch(operation);

	if (response.error) {
		return fetchResponse(response.error.message, { status: errorStatus });
	}

	return ResponseResponse(response.data);
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

		// decode url to what we want
		let url = new URL(request.url);
		console.log({ path: url.pathname })
		let details = url.pathname.split('/');
		console.log({ details })
		let route = details[1];

		// and handle it.
		console.log({ route, details });
		switch (route) {
			case 'users':
				console.log(`Getting user id from ${details[2]}`);
				return handleApiRequest(getIdFromName(details[2]));
			case 'towers':
				console.log(`Getting badge data for ${details[3]}`);

				if (details[3] === 'all') {
					let badges: { badgeids: number[] } = await request.json();
					return handleApiRequest(getAllTowerData(parseInt(details[2]), badges.badgeids));
				}

				return handleApiRequest(getTowerData(parseInt(details[3]), parseInt(details[2])));

			default:
				return new Response('Not Found', { status: 404 });
		}
	},
} satisfies ExportedHandler<Env>;
