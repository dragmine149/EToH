import { fetchRequest } from "./wrappers";

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


export async function processResponse<T>(url: string, user_id: number) {
	console.log("Processing response for URL:", url);

	let response = await tryCatch(fetch(fetchRequest(url)));
	if (response.error) {
		console.error("Failed to fetch badge data:", response.error);
		return DataResponse.FetchFailed(response.error.message);
	}

	if (response.data.status == 204) {
		return DataResponse.UserNoHasBadge(user_id);
	}

	console.log("Processing badge data to requested format");

	let data = await tryCatch<T>(response.data.json());
	if (data.error) {
		console.error("Failed to parse badge data:", data.error);
		return DataResponse.ParseJsonFailed(data.error.message);
	}

	console.log("Badge data processed successfully");

	return data.data;
}

enum DataResponses {
	FetchFailed = 400,
	BadgeNotFound = 404,
	UserFound = 200,
	UserNotFound = 404,
	UserHasBadge = 200,
	UserNoHasBadge = 200,
	UserMassBadgeRequest = 202,
	ParseJsonFailed = 500,
	URLParseFailed = 500,
	APIDoesntExist = 501,
	Unknown = 500
}

export class DataResponse {
	static __makeResponse(json: any, status: number) {
		console.log("Making response with information:", { json, status });
		return Response.json(json, { status });
	}

	static FetchFailed(...error_details: any[]): Response {
		return this.__makeResponse({
			error: `Failed to fetch data. Please try again later as this is an issue with roblox.`,
			error_details
		}, DataResponses.FetchFailed)
	}

	static BadgeNotFound(id: number): Response {
		return this.__makeResponse({
			error: `Badge with this id (${id}) could not be found.`,
		}, DataResponses.BadgeNotFound)
	}

	static UserFound(data: any): Response {
		return this.__makeResponse(data, DataResponses.UserFound);
	}

	static UserNotFoundId(id: number): Response {
		return this.__makeResponse({
			error: `User with this id (${id}) could not be found.`,
		}, DataResponses.UserNotFound)
	}

	static UserNotFoundName(name: string): Response {
		return this.__makeResponse({
			error: `User with this name (${name}) could not be found.`,
		}, DataResponses.UserNotFound)
	}

	static UserHasBadge(data: any): Response {
		return this.__makeResponse(data, DataResponses.UserHasBadge);
	}

	static UserNoHasBadge(id: number): Response {
		return this.__makeResponse({
			error: `User with id (${id}) does not have this badge. OR the badge does not exist (user can't have a badge that doesn't exist).`,
			date: 0
		}, DataResponses.UserNoHasBadge);
	}

	static UserNoHasBadges(id: number): Response {
		return this.__makeResponse({
			error: `User with id (${id}) does not have these badges. OR the badge does not exist (user can't have a badge that doesn't exist).`,
		}, DataResponses.UserNoHasBadge);
	}

	static UserMassBadgeRequest(): Response {
		return this.__makeResponse({
			message: `User has requested multiple badges.`,
		}, DataResponses.UserMassBadgeRequest);
	}

	static ParseJsonFailed(error_message: string): Response {
		return this.__makeResponse({
			error: `Failed to parse JSON data.`,
			error_details: error_message
		}, DataResponses.ParseJsonFailed);
	}

	static APIDoesntExist(): Response {
		return this.__makeResponse({
			error: `API does not exist.`,
		}, DataResponses.APIDoesntExist);
	}

	static URLParseFailed(error_message: any): Response {
		return this.__makeResponse({
			error: `Failed to parse URL.`,
			error_details: error_message
		}, DataResponses.URLParseFailed);
	}

	static Unknown(message: string): Response {
		return this.__makeResponse({
			error: `Unknown error occurred. Please try again later and make an issue on github if the issue persists.`,
			error_details: message
		}, DataResponses.Unknown);
	}
}
