import { tryCatch } from "../utils";
import { fetchRequest } from "../wrappers";

/**
* Gets the id of an user
* @param name The roblox USERNAME (not display name) of the user to get
* @returns The ID of the user
*/
export async function getIdFromName(name: string): Promise<Response> {
	// if we have no name, then return as there is no reason for us to do anything.
	if (!name) {
		return new Response('User not Found', { status: 404 });
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
	let data = await tryCatch<RobloxUserIDResponse>(response.data.json());
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

export async function getNameFromId(id: number): Promise<Response> {
	if (!id) {
		return new Response('Invalid ID, aka user not found.', { status: 404 });
	}

	let response = await tryCatch(fetch(fetchRequest(`https://users.roblox.com/v1/users/${id}`, {
		method: 'GET'
	})));

	if (response.error) {
		return new Response(`Failed to fetch user data: ${response.error.message}`, { status: 500 });
	}

	let data = await tryCatch<RobloxUser>(response.data.json());
	if (data.error || data.data.errors) {
		return new Response(`Failed to fetch user data: Invalid user id`, { status: 500 });
	}

	return Response.json({
		name: data.data.name
	});
}
