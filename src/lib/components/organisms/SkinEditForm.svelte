<script lang="ts">
  import { goto } from "$app/navigation";
  import { API_BASE, MOD_TYPE_OPTIONS, apiFetch, mediaUrl, modTypeLabel, type Character, type Session, type Skin } from "$lib/api";
  import CharacterCombobox from "$lib/components/molecules/CharacterCombobox.svelte";
  import MarkdownContent from "$lib/components/molecules/MarkdownContent.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";

  export let skin: Skin;
  export let characters: Character[] = [];
  export let session: Session;

  let title = skin.title;
  let version = skin.version || "1.0.0";
  let contentKind: "mod" | "plugin" = skin.contentKind || "mod";
  let modType = skin.modType || "complete_skin";
  let description = skin.description || "";
  let characterId = skin.character?.id ?? "";
  let tags = skin.tags.join(", ");
  let pluginDependencies = skin.pluginDependencies?.join(", ") ?? "";
  let videos = skin.videos?.length ? skin.videos.map((video) => ({ label: video.label, url: video.url })) : [{ label: "", url: "" }];
  let links = skin.links?.length ? skin.links.map((link) => ({ label: link.label, url: link.url, kind: link.kind })) : [{ label: "", url: "", kind: "external" }];
  let images = skin.images ?? [];
  let files = skin.files ?? [];
  let newImages: FileList | null = null;
  let newFiles: FileList | null = null;
  let error = "";
  let message = "";
  let saving = false;
  let deleting = false;

  $: cleanVideos = videos.map((video) => ({ label: video.label.trim() || "Video preview", url: video.url.trim() })).filter((video) => video.url);
  $: cleanLinks = links.map((link) => ({ label: link.label.trim() || "Source", url: link.url.trim(), kind: link.kind })).filter((link) => link.url);
  $: tagList = tags.split(",").map((tag) => tag.trim()).filter(Boolean);
  $: pluginDependencyList = pluginDependencies.split(",").map((item) => item.trim()).filter(Boolean);
  $: selectedCharacter = contentKind === "plugin" ? undefined : characters.find((character) => character.id === characterId);
  $: primaryImage = images[0];
  $: newImageFiles = Array.from(newImages ?? []);
  $: newArchiveFiles = Array.from(newFiles ?? []);

  function addVideo() {
    if (videos.length < 4) videos = [...videos, { label: "", url: "" }];
  }

  function removeVideo(index: number) {
    videos = videos.filter((_, itemIndex) => itemIndex !== index);
    if (!videos.length) videos = [{ label: "", url: "" }];
  }

  function addLink() {
    if (links.length < 12) links = [...links, { label: "", url: "", kind: "external" }];
  }

  function removeLink(index: number) {
    links = links.filter((_, itemIndex) => itemIndex !== index);
    if (!links.length) links = [{ label: "", url: "", kind: "external" }];
  }

  async function deleteImage(id: string) {
    error = "";
    try {
      await apiFetch(`/skins/${skin.id}/images/${id}`, { method: "DELETE" }, session.token);
      images = images.filter((image) => image.id !== id);
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not remove image";
    }
  }

  async function deleteFile(id: string) {
    error = "";
    try {
      await apiFetch(`/skins/${skin.id}/files/${id}`, { method: "DELETE" }, session.token);
      files = files.filter((file) => file.id !== id);
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not remove file";
    }
  }

  async function uploadNewFiles() {
    const form = new FormData();
    for (const file of Array.from(newImages ?? [])) form.append("images", file);
    for (const file of Array.from(newFiles ?? [])) form.append("files", file);
    if ([...Array.from(newImages ?? []), ...Array.from(newFiles ?? [])].length === 0) return;
    await apiFetch(`/skins/${skin.id}/upload`, { method: "POST", body: form }, session.token);
  }

  async function save() {
    if (saving) return;
    saving = true;
    error = "";
    message = "";
    try {
      await apiFetch<{ skin: Skin }>(`/skins/${skin.id}`, {
        method: "PATCH",
        body: JSON.stringify({
          title: title.trim(),
          version: version.trim() || "1.0.0",
          contentKind,
          modType,
          description,
          characterId: contentKind === "plugin" ? null : characterId,
          tags: tagList,
          pluginDependencies: pluginDependencyList,
          links: cleanLinks,
          videos: cleanVideos
        })
      }, session.token);
      await uploadNewFiles();
      message = "Skin updated.";
      await goto(`/skins/${skin.slug}`);
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not update skin";
      saving = false;
    }
  }

  async function deleteSkin() {
    if (deleting) return;
    const confirmed = window.confirm(`Delete "${skin.title}" permanently? Uploaded files will be removed from the server.`);
    if (!confirmed) return;
    deleting = true;
    error = "";
    try {
      await apiFetch(`/skins/${skin.id}`, { method: "DELETE" }, session.token);
      await goto("/");
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not delete skin";
      deleting = false;
    }
  }

  function formatBytes(bytes: number) {
    if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<form class="grid gap-5" on:submit|preventDefault={save}>
  <Card class="p-5">
    <div class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
      <div>
        <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Edit</p>
        <h1 class="mt-1 text-3xl font-black tracking-tight">{skin.title}</h1>
        <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">Update the listing, media, obtain methods, and preview before saving.</p>
      </div>
      <Button href={`/skins/${skin.slug}`} variant="ghost" size="sm">Cancel</Button>
    </div>
  </Card>

  <div class="grid items-start gap-5 xl:grid-cols-[minmax(0,1fr)_380px]">
    <div class="grid gap-5">
      <Card class="p-5">
        <div class="mb-5 flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <h2 class="text-lg font-bold">Mod details</h2>
            <p class="mt-1 text-sm text-muted-foreground">Use Markdown for headings, lists, links, bold text, and code blocks.</p>
          </div>
          <span class="rounded-full border border-white/10 bg-background/55 px-3 py-1 text-xs font-bold text-muted-foreground">{description.length}/4000</span>
        </div>
        <div class="grid gap-4 sm:grid-cols-2">
          <Label>Title<Input bind:value={title} placeholder="Gear Five recolor" /></Label>
          <Label>Version<Input bind:value={version} placeholder="1.0.0" /></Label>
          <Label>
            Browser tab
            <select class="h-10 rounded-md border border-white/12 bg-background/55 px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring" bind:value={contentKind}>
              <option value="mod">Mods</option>
              <option value="plugin">Plugins</option>
            </select>
          </Label>
          {#if contentKind === "mod"}
          <Label>
            Mod type
            <select class="h-10 rounded-md border border-white/12 bg-background/55 px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring" bind:value={modType}>
              {#each MOD_TYPE_OPTIONS.filter((option) => option.value) as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </Label>
          <Label class="sm:col-span-2">Character<CharacterCombobox {characters} bind:value={characterId} valueKey="id" includeAll={false} /></Label>
          {/if}
          <Label class="sm:col-span-2">
            Description
            <textarea class="min-h-44 w-full rounded-md border border-white/12 bg-background/55 px-3 py-2 text-sm leading-6 text-foreground shadow-sm outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring" maxlength="4000" bind:value={description}></textarea>
          </Label>
        </div>
      </Card>

      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-lg font-bold">Media</h2>
          <p class="mt-1 text-sm text-muted-foreground">Manage external videos and screenshots. Videos are embedded from the source, never hosted here.</p>
        </div>

        <div class="mb-5 grid gap-3">
          <div class="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h3 class="text-sm font-black">External videos</h3>
              <p class="mt-1 text-xs leading-5 text-muted-foreground">YouTube or Vimeo links appear before screenshots on the detail page.</p>
            </div>
            <span class="w-fit rounded-full border border-white/12 bg-background/55 px-2.5 py-1 text-xs font-black text-muted-foreground">{cleanVideos.length} active</span>
          </div>
          {#each videos as video, index}
            <div class="grid gap-3 rounded-md border border-white/10 bg-background/35 p-3 md:grid-cols-[minmax(140px,0.65fr)_minmax(220px,1.35fr)_auto] md:items-end">
              <Label>Label<Input bind:value={video.label} placeholder="Gameplay preview" /></Label>
              <Label>URL<Input bind:value={video.url} placeholder="https://www.youtube.com/watch?v=..." /></Label>
              <div class="flex h-10 items-center justify-end">
                {#if videos.length > 1}
                  <button class="h-9 rounded-md border border-white/12 px-3 text-xs font-bold text-muted-foreground hover:bg-white/10 hover:text-foreground" type="button" on:click={() => removeVideo(index)}>Remove</button>
                {:else}
                  <span class="hidden h-9 w-[78px] md:block"></span>
                {/if}
              </div>
            </div>
          {/each}
          <Button variant="outline" type="button" on:click={addVideo}>Add another video</Button>
        </div>

        <div class="grid gap-4 rounded-lg border border-dashed border-white/14 bg-background/45 p-4">
          <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <p class="text-sm font-black">Preview images</p>
              <p class="mt-1 text-xs text-muted-foreground">Remove old screenshots or add new ones.</p>
            </div>
            <label class="inline-flex h-10 cursor-pointer items-center justify-center rounded-md border border-white/14 bg-background/70 px-4 text-sm font-bold text-foreground hover:bg-white/10">
              Choose images
              <input class="sr-only" bind:files={newImages} type="file" accept="image/png,image/jpeg,image/webp,image/gif" multiple />
            </label>
          </div>

          {#if images.length}
            <div class="grid gap-3 sm:grid-cols-2">
              {#each images as image}
                <div class="overflow-hidden rounded-lg border border-white/10 bg-background/55">
                  <div class="aspect-video bg-black/60">
                    <img class="h-full w-full object-cover" src={mediaUrl(image.url)} alt={image.alt || skin.title} />
                  </div>
                  <div class="flex items-center justify-between gap-3 px-3 py-2 text-xs">
                    <span class="min-w-0 truncate font-bold text-foreground">{image.alt || "Image"}</span>
                    <button class="shrink-0 font-black text-red-300 hover:text-red-200" type="button" on:click={() => deleteImage(image.id)}>Remove</button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}

          {#if newImageFiles.length}
            <div class="grid gap-2 text-xs sm:grid-cols-2">
              {#each newImageFiles as file}
                <div class="flex items-center justify-between gap-3 rounded-md border border-white/10 bg-background/35 px-3 py-2">
                  <span class="min-w-0 truncate font-bold">{file.name}</span>
                  <span class="shrink-0 text-muted-foreground">{formatBytes(file.size)}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </Card>

      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-lg font-bold">Credits</h2>
          <p class="mt-1 text-sm text-muted-foreground">Creator assignment is handled by the original listing and moderation tools.</p>
        </div>
        <div class="grid gap-4 sm:grid-cols-2">
          <Label>Tags<Input bind:value={tags} placeholder="wano, recolor, coat" /></Label>
          {#if contentKind === "mod"}
            <Label>Plugin dependencies<Input bind:value={pluginDependencies} placeholder="lua-core, costume-api" /></Label>
          {/if}
        </div>
      </Card>

      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-lg font-bold">Obtain</h2>
          <p class="mt-1 text-sm text-muted-foreground">Manage public obtain links and hosted ZIP files from one place.</p>
        </div>
        <div class="grid gap-6">
          <section class="grid gap-3">
            <div class="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
              <div>
                <h3 class="text-base font-black">External links</h3>
                <p class="mt-1 text-xs leading-5 text-muted-foreground">Patreon, Drive, Nexus, mirrors, creator pages, or any public source.</p>
              </div>
              <span class="w-fit rounded-full border border-white/12 bg-background/55 px-2.5 py-1 text-xs font-black text-muted-foreground">{cleanLinks.length} active</span>
            </div>

            <div class="grid gap-2">
              {#each links as link, index}
                <div class="grid gap-3 rounded-md border border-white/10 bg-background/35 p-3 lg:grid-cols-[125px_minmax(130px,0.75fr)_minmax(220px,1.5fr)_auto] lg:items-end">
                  <Label>
                    Type
                    <select class="h-10 rounded-md border border-white/12 bg-background/55 px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring" bind:value={link.kind}>
                      <option value="patreon">Patreon</option>
                      <option value="kofi">Ko-fi</option>
                      <option value="nexus">Nexus</option>
                      <option value="drive">Drive</option>
                      <option value="mirror">Mirror</option>
                      <option value="external">External</option>
                    </select>
                  </Label>
                  <Label>Label<Input bind:value={link.label} placeholder="Patreon" /></Label>
                  <Label>URL<Input bind:value={link.url} placeholder="https://..." /></Label>
                  <div class="flex h-10 items-center justify-between gap-3 lg:justify-end">
                    <span class="text-xs font-bold text-muted-foreground lg:hidden">Link {index + 1}</span>
                    {#if links.length > 1}
                      <button class="h-9 rounded-md border border-white/12 px-3 text-xs font-bold text-muted-foreground hover:bg-white/10 hover:text-foreground" type="button" on:click={() => removeLink(index)}>Remove</button>
                    {:else}
                      <span class="hidden h-9 w-[78px] lg:block"></span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>

            <Button variant="outline" type="button" on:click={addLink}>Add another link</Button>
          </section>

          <div class="flex items-center gap-3">
            <div class="h-px flex-1 bg-border"></div>
            <span class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">or host it here</span>
            <div class="h-px flex-1 bg-border"></div>
          </div>

          <section class="grid gap-3">
            <div class="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
              <div>
                <h3 class="text-base font-black">Hosted ZIP</h3>
                <p class="mt-1 text-xs leading-5 text-muted-foreground">Use when users should download the file directly from this hub.</p>
              </div>
              <span class="w-fit rounded-full border border-white/12 bg-background/55 px-2.5 py-1 text-xs font-black text-muted-foreground">{files.length + newArchiveFiles.length} file{files.length + newArchiveFiles.length > 1 ? "s" : ""}</span>
            </div>

            {#if files.length}
              <div class="grid gap-2">
                {#each files as file}
                  <div class="flex items-center justify-between gap-3 rounded-md border border-white/10 bg-background/35 px-3 py-2 text-sm">
                    <a class="min-w-0 truncate font-bold text-primary hover:underline" href={`${API_BASE}/files/${file.id}/download`}>{file.fileName}</a>
                    <span class="shrink-0 text-xs text-muted-foreground">{formatBytes(file.sizeBytes)}</span>
                    <button class="shrink-0 text-xs font-black text-red-300 hover:text-red-200" type="button" on:click={() => deleteFile(file.id)}>Remove</button>
                  </div>
                {/each}
              </div>
            {/if}

            <div class="grid gap-3 rounded-md border border-dashed border-white/14 bg-background/35 p-4 sm:grid-cols-[1fr_auto] sm:items-center">
              <div>
                <p class="text-sm font-black">Add ZIP files</p>
                <p class="mt-1 text-xs leading-5 text-muted-foreground">ZIP only. 20 MB max per file.</p>
              </div>
              <input class="w-full rounded-md border border-white/12 bg-background/55 px-3 py-2 text-sm text-foreground file:mr-3 file:rounded-md file:border-0 file:bg-primary file:px-3 file:py-1.5 file:text-sm file:font-medium file:text-primary-foreground sm:w-72" bind:files={newFiles} type="file" accept=".zip,application/zip" multiple />
            </div>

            {#if newArchiveFiles.length}
              <div class="grid gap-2 text-xs sm:grid-cols-2">
                {#each newArchiveFiles as file}
                  <div class="flex items-center justify-between gap-3 rounded-md border border-white/10 bg-background/35 px-3 py-2">
                    <span class="min-w-0 truncate font-bold">{file.name}</span>
                    <span class="shrink-0 text-muted-foreground">{formatBytes(file.size)}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </section>
        </div>
      </Card>

      <Card class="p-5">
        <div class="mb-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 class="text-lg font-bold">Live preview</h2>
            <p class="mt-1 text-sm text-muted-foreground">Check how this mod will read after saving.</p>
          </div>
        </div>
        <div class="grid gap-4">
          {#if primaryImage}
            <div class="overflow-hidden rounded-lg border border-white/10 bg-card/70">
              <div class="relative aspect-video overflow-hidden bg-black">
                <img class="absolute inset-0 h-full w-full scale-110 object-cover opacity-30 blur-2xl" src={mediaUrl(primaryImage.url)} alt="" aria-hidden="true" />
                <div class="absolute inset-0 bg-black/52"></div>
                <img class="relative z-10 h-full w-full object-contain" src={mediaUrl(primaryImage.url)} alt={primaryImage.alt || title} />
              </div>
            </div>
          {/if}
          <div class="rounded-lg border border-white/10 bg-background/45 p-5">
            <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">{selectedCharacter?.displayName ?? skin.character?.displayName ?? "Plugin"}</p>
            <div class="mt-2 flex flex-wrap items-end gap-3">
              <h3 class="text-3xl font-black tracking-tight">{title || "Untitled mod"}</h3>
              <span class="rounded-md border border-white/12 bg-background/55 px-2 py-1 text-xs font-black text-muted-foreground">v{version || "1.0.0"}</span>
              <span class="rounded-md border border-primary/30 bg-primary/10 px-2 py-1 text-xs font-black text-primary">{modTypeLabel(modType)}</span>
            </div>
            <div class="mt-4">
              <MarkdownContent value={description} />
            </div>
          </div>
        </div>
      </Card>
    </div>

    <aside class="grid content-start gap-5 xl:sticky xl:top-24">
      <Card class="p-5">
        <Button type="submit" class="w-full" disabled={saving}>{saving ? "Saving..." : "Save changes"}</Button>
        {#if message}
          <p class="mt-3 text-center text-xs text-emerald-300">{message}</p>
        {/if}
        {#if error}
          <p class="mt-3 text-center text-xs text-red-300">{error}</p>
        {/if}
      </Card>

      {#if skin.status === "rejected"}
        <Card class="border-red-400/25 bg-red-500/10 p-5">
          <h2 class="text-lg font-bold text-red-100">Danger zone</h2>
          <p class="mt-2 text-sm leading-6 text-red-100/75">Deleting a rejected mod is permanent and removes uploaded files from the server.</p>
          <Button type="button" variant="destructive" class="mt-4 w-full" disabled={deleting} on:click={deleteSkin}>
            {deleting ? "Deleting..." : "Delete mod"}
          </Button>
        </Card>
      {/if}
    </aside>
  </div>
</form>
