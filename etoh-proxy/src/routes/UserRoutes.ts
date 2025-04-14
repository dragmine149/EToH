import { DataResponse } from '../utils';
import { getNameFromId, getIdFromName } from '../apis/users';

export class UserRoutes {
	static async handle(response: BindingsResponse) {
		const { user_id, username, option, error } = response;

		if (Object.keys(error).length > 0) {
			return DataResponse.URLParseFailed(error);
		}

		switch (option) {
			case 'name':
				console.log(`Getting user name from ${user_id}`);
				return getNameFromId(user_id);

			case '':
			case 'id':
				console.log(`Getting user id from ${username}`);
				return getIdFromName(username);

			default:
				return DataResponse.APIDoesntExist();
		}
	}
}
