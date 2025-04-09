export const ENDPOINTS = {
	"users": [
		{
			method: 'GET',
			path: '/users/{username}',
			description: 'Get user ID from username',
			parameters: [
				{
					name: 'username',
					type: 'string',
					required: true,
					description: 'The username of the user to get the ID for'
				}
			],
			responses: [
				{
					code: 404,
					description: 'Username is invalid or user does not exist',
				},
				{
					code: 500,
					description: 'Failed to fetch/parse user data (Probably an issue on roblox end)'
				},
				{
					code: 200,
					description: 'User data fetched successfully',
					model: {
						id: {
							type: 'number',
							description: 'The Roblox user ID'
						}
					}
				}
			]
		},
		{
			method: 'GET',
			path: '/users/{userid}/name',
			description: 'Get user name from user ID',
			parameters: [
				{
					name: 'userid',
					type: 'number',
					required: true,
					description: 'The Roblox user ID to get the data for'
				}
			],
			responses: [
				{
					code: 404,
					description: 'User ID is invalid or user does not exist',
				},
				{
					code: 500,
					description: 'Failed to fetch/parse user data (Probably an issue on roblox end)'
				},
				{
					code: 200,
					description: 'User data fetched successfully',
					model: {
						name: {
							type: 'string',
							description: 'The Roblox username'
						}
					}
				}
			]
		}
	],
	"badges": [
		{
			method: 'GET',
			path: '/towers/{userid}/{badgeid}',
			description: 'Get badge award date from user ID and badge ID',
			parameters: [
				{
					name: 'userid',
					type: 'number',
					required: true,
					description: 'The Roblox user ID to get the data for'
				},
				{
					name: 'badgeid',
					type: 'number',
					required: true,
					description: 'The Roblox badge ID to get the data for'
				}
			],
			responses: [
				{
					code: 404,
					description: 'User ID or badge ID is invalid or user does not have the badge',
				},
				{
					code: 500,
					description: 'Failed to fetch/parse badge data (Probably an issue on roblox end)'
				},
				{
					code: 200,
					description: 'Badge data fetched successfully',
					model: {
						awardedDate: {
							type: 'number',
							description: 'The timestamp when the badge was awarded'
						}
					}
				}
			]
		},
		{
			method: 'GET',
			path: '/towers/{userid}/all',
			description: 'Get all badge award dates from user ID',
			streamed: true,
			parameters: [
				{
					name: 'userid',
					type: 'number',
					required: true,
					description: 'The Roblox user ID to get the data for'
				},
				{
					name: 'badgeids',
					type: 'Array<number>',
					required: true,
					description: 'The Roblox badge ID to get the data for',
					notes: [
						'If the user does not have a badge, it will not be included in the response',
						'This list must be provided as a json in the body. Prvoided it in the url will not work (unless you know how to get it to treat like a body)'
					]
				},
			],
			responses: [
				{
					code: 404,
					description: 'User ID is invalid or user does not exist',
				},
				{
					code: 500,
					description: 'Failed to fetch/parse user data (Probably an issue on roblox end)'
				},
				{
					code: 200,
					description: 'User data fetched successfully',
					model: {
						badgeId: {
							type: 'number',
							description: 'The Roblox badge ID'
						},
						awardedDate: {
							type: 'number',
							description: 'The timestamp when the badge was awarded'
						}
					}
				}
			]
		},
	]
};
