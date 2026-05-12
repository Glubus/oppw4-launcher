export const ROLES = {
  USER: "ROLE_USER",
  CREATOR: "ROLE_CREATOR",
  MODERATOR: "ROLE_MODERATOR",
  ADMIN: "ROLE_ADMIN",
  BANNED: "ROLE_BANNED"
} as const;

export type Role = (typeof ROLES)[keyof typeof ROLES];

export const SKIN_STATUSES = ["draft", "pending", "published", "rejected", "hidden"] as const;
export type SkinStatus = (typeof SKIN_STATUSES)[number];

export const OWNERSHIP_TYPES = ["own_work", "community_repost"] as const;
export type OwnershipType = (typeof OWNERSHIP_TYPES)[number];

export const MOD_TYPES = ["complete_skin", "ui", "effects", "audio", "moveset", "misc"] as const;
export type ModType = (typeof MOD_TYPES)[number];

export type Oppw4Character = {
  slug: string;
  name: string;
  displayName: string;
  isDlc: boolean;
  pack: string;
};

export { OPPW4_CHARACTERS } from "./oppw4Characters";
