import { DataResponse, tryCatch } from "../utils";
import { fetchRequest } from "../wrappers";

/**
* Gets the id of an user
* @param name The roblox USERNAME (not display name) of the user to get
* @returns The ID of the user
*/
export async function getIdFromName(name: string): Promise<Response> {
	// if we have no name, then return as there is no reason for us to do anything.
	if (!name) {
		return DataResponse.UserNotFoundName(name);
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
		return DataResponse.FetchFailed(response.error.message);
	}


	// decode the id from the data
	let data = await tryCatch<RobloxUserIDResponse>(response.data.json());
	if (data.error) {
		return DataResponse.ParseJsonFailed(data.error.message);
	}

	let rbx_data = data.data.data;

	// just make sure we have data
	if (rbx_data?.length > 0) {
		return DataResponse.UserFound({
			id: rbx_data[0].id
		});
	}

	return DataResponse.UserNotFoundName(name);
}

export async function getNameFromId(id: number): Promise<Response> {
	if (!id) {
		return DataResponse.UserNotFoundId(id);
	}

	let response = await tryCatch(fetch(fetchRequest(`https://users.roblox.com/v1/users/${id}`, {
		method: 'GET'
	})));

	if (response.error) {
		return DataResponse.FetchFailed(response.error.message);
	}

	let data = await tryCatch<RobloxUser>(response.data.json());
	if (data.error) {
		return DataResponse.ParseJsonFailed(data.error.message);
	}

	if (data.data.errors) {
		console.warn(data.data.errors);
		return DataResponse.UserNotFoundId(id);
	}

	return DataResponse.UserFound({
		name: data.data.name
	});
}
