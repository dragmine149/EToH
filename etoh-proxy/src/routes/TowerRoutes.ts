import { DataResponse } from '../utils';
import { getTowerData, getAllTowerData, compareBadges } from '../apis/badges';

export class TowerRoutes {
	static async handle(response: BindingsResponse, request: Request) {
		const { user_id, option, badge_1, badge_2 } = response;

		console.log(`Getting badge data for ${user_id}`, { option, badge_1, badge_2 });

		switch (option) {
			case 'earliest':
				return compareBadges(user_id, badge_1, badge_2);
			case 'all':
				const badges: { badgeids: number[] } = await request.json();
				return getAllTowerData(user_id, badges.badgeids);
			case 'badge':
				return getTowerData(user_id, badge_1);
			default:
				return DataResponse.APIDoesntExist();
		}
	}
}
