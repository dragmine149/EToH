/*eslint no-unused-vars: "error"*/

import Dexie, { type EntityTable, type InsertType, type Table } from "dexie";

interface BadgeData {
  badgeId: number;
  userId: number;
  date: number;
}
interface UserData {
  id: number;
  name: string;
  display: string;
  past: string[];
  last: number;
}

type UserTable = Table<UserData, number, InsertType<UserData, "id">>;


const etohDB = new Dexie("EToH") as Dexie & {
  badges: EntityTable<BadgeData, 'badgeId'>,
  users: EntityTable<UserData, 'id'>
};
etohDB.version(1).stores({
  badges: `[badgeId+userId], badgeId, userId`,
  users: `&id, name, display, past, last`
})

export { etohDB };
export type { UserData, BadgeData, UserTable };
