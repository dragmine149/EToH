import { tryCatch } from '../utils';
import { fetchRequest } from '../wrappers';

export function processDate(date: string) {
	let d = new Date(date);
	return d.getTime();
}

/**
* Gets the awarded date of a badge
* @param badge_id The id of the badge
* @param user_id The id of the user
* @returns The date/time the badge got awarded
*/
export async function getTowerData(user_id: number, badge_id: number) {
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
		return Response.json({ awardedDate: processDate(rbx_data.awardedDate) });
	}

	return new Response(`Badge not found`, { status: 404 });
}

/**
* Gets the completion date of every badge assigned. If a badge has no date, nothing is returned.
* @param user_id The roblox user ID
* @param badges The list of badges to get
* @returns A streamable response of the badges upon receiving them from roblox.
*/
export async function getAllTowerData(user_id: number, badges: number[]) {
	let chunkSize = 100; // roblox has a limit on the amount of badges you can request at once, although this exact limit is unknown, 100 seems big enough,
	let url = `https://badges.roblox.com/v1/users/${user_id}/badges/awarded-dates`;

	// create new streams and stuff for streaming the data.
	let { readable, writable } = new TransformStream();
	let writer = writable.getWriter();
	let encoder = new TextEncoder();

	/**
	* If the request fails once, try again but with half the chunk size.
	* If the request fails twice, wait 2 seconds and then try again.
	* If the request fails thrice, half the chunk size (again), wait 5 seconds and then try again
	* If the request fails fouce, abort
	* Everytime make sure to update the chunksize for all future request, however only wait whilst retrying that one request (after the retry success, continue with 0 wait
	*/

	// Start processing in background but don't await it
	(async () => {
		let retryCount = 0;
		async function on_error(i: number, error: Error) {
			// add a retry and reset the chunk size back.
			retryCount++;
			i -= chunkSize;

			// reduce the chunkSize upon failing x times.
			if (retryCount % 2 == 1) {
				chunkSize = Math.max(1, Math.floor(chunkSize / 2));
			}

			// avoid the one off situation where i becomes negative and breaks things.
			i = Math.min(0, i);

			await writer.write(encoder.encode(`Failed to fetch from server due to: ${error.message}`));
			if (retryCount === 2) await new Promise(resolve => setTimeout(resolve, 2000));
			if (retryCount === 3) await new Promise(resolve => setTimeout(resolve, 5000));

			return i;
		}

		for (let i = 0; i < badges.length; i += chunkSize) {
			if (retryCount > 3) {
				// ok, give up.
				await writer.abort(new Error('Failed after 4 retries'));
				return;
			}

			// chunk and generate the other part of the url.
			let chunk = badges.slice(i, i + chunkSize);
			let badge_search = chunk.map(badge => badge.toString()).join(',');

			// send the request to the server
			let response = await tryCatch(fetch(fetchRequest(`${url}?badgeIds=${badge_search}`, {
				headers: {
					'Content-Type': 'application/json'
				},
			})));

			// process errors and the data
			if (response.error) {
				i = await on_error(i, response.error);
				continue;
			}

			let data = await tryCatch<RobloxBadgeResponse>(response.data.json());

			if (data.error) {
				i = await on_error(i, data.error);
				continue;
			}

			// send all data back to the clients
			for (let badge of data.data.data) {
				if (badge.awardedDate == undefined) {
					continue;
				}

				badge.date = processDate(badge.awardedDate);
				delete badge.awardedDate;
				await writer.write(encoder.encode(JSON.stringify(badge) + '\n'));
			}
			retryCount = 0; // no retry needed YAY
		}

		await writer.close();
	})();

	// Return response immediately while processing continues in background
	return new Response(readable, {
		headers: {
			'Content-Type': 'application/x-ndjson',
			'Transfer-Encoding': 'chunked'
		}
	});
}

export async function compareBadges(user_id: number, badge_1: number, badge_2: number) {
	let url = `https://badges.roblox.com/v1/users/${user_id}/badges/awarded-dates?badgeIds=${badge_1},${badge_2}`;
	console.log(`Fetching data`);
	let response = await tryCatch(fetch(fetchRequest(url, {
		headers: {
			'Content-Type': 'application/json'
		},
	})));

	console.log("Data fetched");

	if (response.error) {
		return new Response(`Failed to fetch badge data: ${response.error.message}`, { status: 500 });
	}

	console.log("Parsing data");

	let data = await tryCatch<RobloxBadgeResponse>(response.data.json());
	if (data.error) {
		return new Response(`Failed to parse badge data: ${data.error.message}`, { status: 500 });
	}

	console.log("Data parsed");
	console.log(`Data: `, data.data.data);

	let rbx_data = data.data.data;
	let return_data = {
		earliest: -1,
		data: rbx_data
	};

	if (rbx_data.length <= 0) {
		return Response.json(return_data);
	}
	rbx_data = rbx_data.map(badge => {
		if (badge.awardedDate == undefined) {
			return badge;
		}

		badge.date = processDate(badge.awardedDate);
		delete badge.awardedDate;
		return badge;
	});

	let date = Math.min(...rbx_data.map(badge => badge.date));
	let earliest = rbx_data.find(v => v.date == date);

	if (earliest == undefined) {
		return new Response(`Failed to get earliest badge somehow. Please try again or report a bug`, { status: 501 });
	}

	return_data.earliest = earliest.badgeId;
	return Response.json(return_data, { status: 200 });

}
