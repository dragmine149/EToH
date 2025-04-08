declare module '*.html' {
	const content: string;
	export default content;
}

declare module '*.css' {
	const content: string;
	export default content;
}

type RobloxUser = {
	description: string;
	created: string;
	isBanned: boolean;
	externalAppDisplayName: (string | null);
	hasVerifiedBadge: boolean;
	id: number;
	name: string;
	displayName: string;
	errors?: {
		code: number;
		message: string;
	}[];
}

type RobloxUserID = {
	requestedUsername: string;
	hasVerifiedBadge: boolean;
	id: number;
	name: string;
	displayName: string;
}

type RobloxUserIDResponse = {
	data: RobloxUserID[];
}

type BadgeResponse = {
	badgeId: number;
	awardedDate?: string;
	date: number;
}

type RobloxBadgeResponse = {
	data: BadgeResponse[];
}

type Success<T> = {
	data: T;
	error: null;
};

type Failure<E> = {
	data: null;
	error: E;
};

type Result<T, E = Error> = Success<T> | Failure<E>;

type Endpoint = {
	method: string;
	path: string;
	title: string;
	description: string;
	example: string;
	response: string;
}
