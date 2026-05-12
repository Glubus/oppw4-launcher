export const API_BASE = import.meta.env.VITE_API_BASE ?? "https://oppw4.prism.am/api";
const PUBLIC_BASE = API_BASE.endsWith("/api") ? API_BASE.slice(0, -4) : API_BASE;

export type Character = {
  id: string;
  slug: string;
  displayName: string;
  isDlc: boolean;
  pack: string;
};

export type SocialLink = {
  label: string;
  url: string;
  kind: string;
};

export type SkinImage = {
  id: string;
  url: string;
  alt?: string | null;
  sortOrder?: number;
};

export type SkinVideo = {
  id: string;
  url: string;
  label: string;
  provider: string;
  sortOrder?: number;
};

export type SkinFile = {
  id: string;
  fileName: string;
  url: string;
  sizeBytes: number;
};

export type SkinLink = {
  id: string;
  label: string;
  url: string;
  kind: string;
  clickCount: number;
};

export type Skin = {
  id: string;
  title: string;
  slug: string;
  version: string;
  modType: string;
  description: string;
  character: Character;
  submittedByUserId?: string | null;
  creditedUserId?: string | null;
  creditedUsername?: string | null;
  creditedSocialLinks?: SocialLink[];
  externalCreatorSlug?: string | null;
  externalCreatorName?: string | null;
  externalCreatorUrl?: string | null;
  ownershipType: string;
  status: string;
  tags: string[];
  images?: SkinImage[];
  videos?: SkinVideo[];
  files?: SkinFile[];
  links?: SkinLink[];
  viewedCount: number;
  redirectionCount: number;
  voteCount: number;
  isPinned?: boolean;
  createdAt: string;
};

export type PublicUser = {
  id: string;
  username: string;
  email?: string;
  roles?: string[];
  socialLinks: SocialLink[];
  stats?: {
    modCount: number;
    downloadCount: number;
    upvoteCount: number;
  };
  canManagePins?: boolean;
  createdAt: string;
};

export const MOD_TYPE_OPTIONS = [
  { value: "", label: "All mod types" },
  { value: "complete_skin", label: "Complete skin" },
  { value: "ui", label: "UI / portraits" },
  { value: "effects", label: "Effects" },
  { value: "audio", label: "Audio" },
  { value: "moveset", label: "Moveset" },
  { value: "misc", label: "Misc" }
];

export function modTypeLabel(value?: string | null) {
  return MOD_TYPE_OPTIONS.find((option) => option.value === value)?.label ?? "Complete skin";
}

export function mediaUrl(url: string) {
  if (url.startsWith("http://") || url.startsWith("https://")) return url;
  return `${PUBLIC_BASE}${url}`;
}

export type Session = {
  token: string;
  user: {
    id: string;
    username: string;
    email: string;
    roles: string[];
    socialLinks: SocialLink[];
  };
};

export async function apiFetch<T>(path: string, init: RequestInit = {}, token?: string): Promise<T> {
  if (isTauriRuntime() && !(init.body instanceof FormData)) {
    const { invoke } = await import("@tauri-apps/api/core");
    const data = await invoke<T>("api_request", {
      input: {
        method: init.method ?? "GET",
        path,
        body: typeof init.body === "string" ? init.body : undefined,
        token
      }
    });
    return data;
  }

  const headers = new Headers(init.headers);
  if (!headers.has("content-type") && init.body && !(init.body instanceof FormData)) {
    headers.set("content-type", "application/json");
  }
  if (token) headers.set("authorization", `Bearer ${token}`);

  const response = await fetch(`${API_BASE}${path}`, { ...init, headers });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.error ?? "Request failed");
  }
  return data as T;
}

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
