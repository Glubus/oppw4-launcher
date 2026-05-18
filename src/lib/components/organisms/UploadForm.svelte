<script lang="ts">
  import { onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { MOD_TYPE_OPTIONS, apiFetch, modTypeLabel, type Character, type Session } from "$lib/api";
  import CharacterCombobox from "$lib/components/molecules/CharacterCombobox.svelte";
  import MarkdownContent from "$lib/components/molecules/MarkdownContent.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";
  import { toastStore } from "$lib/stores/toasts";
  import { markdownToPlainText } from "$lib/utils/markdown";

  export let characters: Character[] = [];
  export let session: Session | null = null;

  type ChecklistTone = "green" | "orange" | "red";
  type ChecklistItem = {
    tone: ChecklistTone;
    label: string;
    description: string;
  };

  let message = "";
  let error = "";
  let title = "";
  let version = "1.0.0";
  let contentKind: "mod" | "plugin" = "mod";
  let modType = "complete_skin";
  let description = "";
  let sourceCodeUrl = "";
  let luaModuleName = "";
  let characterId = "";
  let ownershipType = "own_work";
  let externalCreatorName = "";
  let externalCreatorUrl = "";
  let tags = "";
  let pluginDependencies = "";
  let videos = [{ label: "", url: "" }];
  let links = [{ label: "", url: "", kind: "external" }];
  let convertedImages: File[] = [];
  let imagePreviews: Array<{ name: string; size: number; url: string; converted: boolean }> = [];
  let imageStatus = "";
  let files: FileList | null = null;
  let isSubmitting = false;
  let previewMode: "card" | "page" = "card";
  let isDraggingImages = false;
  let isDraggingArchive = false;

  const maxArchiveBytes = 20 * 1024 * 1024;
  const webpQuality = 0.86;

  $: if (contentKind === "mod" && !characterId && characters.length) characterId = String(characters[0].id);
  $: selectedCharacter = contentKind === "plugin"
    ? ({ id: "plugin", slug: "plugin", displayName: "Plugin", isDlc: false, pack: "Plugins" } satisfies Character)
    : characters.find((character) => String(character.id) === String(characterId));
  $: isOwnWork = ownershipType === "own_work";
  $: if (isOwnWork) {
    externalCreatorName = "";
    externalCreatorUrl = "";
  }
  $: cleanLinks = links
    .map((link) => ({ label: link.label.trim() || "Source", url: link.url.trim(), kind: link.kind }))
    .filter((link) => link.url);
  $: cleanVideos = videos
    .map((video) => ({ label: video.label.trim() || "Video preview", url: video.url.trim() }))
    .filter((video) => video.url);
  $: archiveFiles = Array.from(files ?? []);
  $: isPluginUpload = contentKind === "plugin";
  $: titleValue = title.trim();
  $: descriptionValue = description.trim();
  $: descriptionPreview = markdownToPlainText(descriptionValue);
  $: tagList = tags
    .split(",")
    .map((tag) => tag.trim())
    .filter(Boolean);
  $: archiveTooLarge = archiveFiles.find((file) => file.size > maxArchiveBytes);
  $: invalidHostedFile = archiveFiles.find((file) => isPluginUpload ? !file.name.toLowerCase().endsWith(".dll") : !file.name.toLowerCase().endsWith(".zip"));
  $: invalidHostedFileCount = isPluginUpload && archiveFiles.length > 1;
  $: hasObtainMethod = isPluginUpload ? archiveFiles.length === 1 : cleanLinks.length > 0 || archiveFiles.length > 0;
  $: needsExternalCreator = ownershipType !== "own_work";
  $: checklist = buildChecklist(
    session,
    titleValue,
    selectedCharacter,
    archiveTooLarge,
    hasObtainMethod,
    cleanLinks.length,
    needsExternalCreator,
    externalCreatorName.trim(),
    externalCreatorUrl.trim(),
    isOwnWork,
    descriptionValue,
    imagePreviews.length,
    cleanVideos.length,
    tagList.length,
    isPluginUpload,
    sourceCodeUrl.trim(),
    invalidHostedFile,
    invalidHostedFileCount
  );
  $: blockerCount = checklist.filter((item) => item.tone === "red").length;
  $: suggestionCount = checklist.filter((item) => item.tone === "orange").length;
  $: canSubmit = blockerCount === 0 && !isSubmitting;
  $: creatorName = isOwnWork ? session?.user.username : externalCreatorName.trim();
  $: primaryImage = imagePreviews[0];
  $: primaryVideo = cleanVideos[0];

  function buildChecklist(
    currentSession: Session | null,
    currentTitle: string,
    currentCharacter: Character | undefined,
    currentArchiveTooLarge: File | undefined,
    currentHasObtainMethod: boolean,
    currentLinkCount: number,
    currentNeedsExternalCreator: boolean,
    currentExternalCreatorName: string,
    currentExternalCreatorUrl: string,
    currentIsOwnWork: boolean,
    currentDescription: string,
    currentImageCount: number,
    currentVideoCount: number,
    currentTagCount: number,
    currentIsPluginUpload: boolean,
    currentSourceCodeUrl: string,
    currentInvalidHostedFile: File | undefined,
    currentInvalidHostedFileCount: boolean
  ): ChecklistItem[] {
    const items: ChecklistItem[] = [];

    items.push(currentSession
      ? { tone: "green", label: "Account ready", description: `Uploading as ${currentSession.user.username}.` }
      : { tone: "red", label: "Log in required", description: "You must be logged in before submitting a skin." });

    items.push(currentTitle
      ? { tone: "green", label: "Title set", description: "The skin has a searchable name." }
      : { tone: "red", label: "Title required", description: "Add a clear skin title." });

    items.push(currentCharacter
      ? { tone: "green", label: "Character selected", description: currentCharacter.displayName }
      : { tone: "red", label: currentIsPluginUpload ? "Plugin target ready" : "Character required", description: currentIsPluginUpload ? "Plugins are installed globally." : "Choose the character this skin belongs to." });

    if (currentInvalidHostedFile) {
      items.push({ tone: "red", label: currentIsPluginUpload ? "DLL required" : "ZIP required", description: `${currentInvalidHostedFile.name} has the wrong file type.` });
    } else if (currentInvalidHostedFileCount) {
      items.push({ tone: "red", label: "One DLL only", description: "Plugin uploads must contain exactly one DLL." });
    } else if (currentArchiveTooLarge) {
      items.push({ tone: "red", label: "ZIP too large", description: `${currentArchiveTooLarge.name} is larger than 20 MB.` });
    } else if (currentHasObtainMethod) {
      items.push({ tone: "green", label: "Obtain method ready", description: currentLinkCount ? "External link available." : "ZIP upload available." });
    } else {
      items.push({ tone: "red", label: "Obtain method required", description: "Add at least one link or ZIP file." });
    }

    if (currentNeedsExternalCreator && !currentExternalCreatorName) {
      items.push({ tone: "red", label: "Creator credit required", description: "Add the external creator name." });
    } else {
      items.push({ tone: "green", label: "Credits consistent", description: currentIsOwnWork ? "Credit will use your profile." : "External credit is ready." });
    }

    items.push(currentDescription
      ? { tone: "green", label: "Description added", description: "Markdown preview is available." }
      : { tone: "orange", label: "Description recommended", description: "Explain version support, changes, and install notes." });

    items.push(currentImageCount
      ? { tone: "green", label: "Images ready", description: `${currentImageCount} preview image${currentImageCount > 1 ? "s" : ""} added.` }
      : { tone: "orange", label: "Images recommended", description: "Screenshots help users choose faster." });

    items.push(currentVideoCount
      ? { tone: "green", label: "Video preview added", description: "The detail page will show it before images." }
      : { tone: "orange", label: "Video can help", description: "Add a YouTube or Vimeo preview when available." });

    items.push(currentTagCount >= 3
      ? { tone: "green", label: "Tags useful", description: `${currentTagCount} tags added.` }
      : { tone: "orange", label: "More tags can help", description: "Add style, arc, outfit, or version tags." });

    if (currentNeedsExternalCreator) {
      items.push(currentExternalCreatorUrl
        ? { tone: "green", label: "Source URL added", description: "Users can find the original creator." }
        : { tone: "orange", label: "Source URL recommended", description: "Link the original post or creator page when possible." });
    }

    if (currentIsPluginUpload) {
      items.push(currentSourceCodeUrl
        ? { tone: "green", label: "Source code linked", description: "Required for plugin review." }
        : { tone: "red", label: "Source code required", description: "Plugins must link their source code." });
    }

    return items;
  }

  async function submitSkin() {
    if (!canSubmit) {
      error = blockerCount ? "Resolve the red checklist items before submitting." : "";
      return;
    }

    isSubmitting = true;
    error = "";
    message = "";
    try {
      const form = new FormData();
      form.set("title", titleValue);
      form.set("version", version.trim() || "1.0.0");
      form.set("contentKind", contentKind);
      form.set("modType", modType);
      form.set("description", description);
      form.set("sourceCodeUrl", sourceCodeUrl);
      if (contentKind === "plugin" && luaModuleName.trim()) form.set("luaModuleName", luaModuleName.trim());
      form.set("characterId", contentKind === "plugin" ? "" : characterId);
      form.set("ownershipType", ownershipType);
      form.set("externalCreatorName", externalCreatorName);
      form.set("externalCreatorUrl", externalCreatorUrl);
      form.set("tags", tags);
      form.set("pluginDependencies", pluginDependencies);
      form.set("links", JSON.stringify(cleanLinks));
      form.set("videos", JSON.stringify(cleanVideos));

      if (convertedImages.length) {
        for (const image of convertedImages) form.append("images", image);
      }
      for (const file of archiveFiles) form.append("files", file);

      if (contentKind === "plugin") form.set("docs", description);
      const result = await apiFetch<{ skin?: { title: string; status: string; slug: string }; plugin?: { title: string; status: string; slug: string } }>(contentKind === "plugin" ? "/plugins" : "/skins", {
        method: "POST",
        body: form
      }, session?.token);
      const created = result.plugin ?? result.skin!;
      if (created.status === "pending" || !created.slug) {
        toastStore.push("Your mod was submitted and is waiting for review.", "success");
        await goto("/");
        return;
      }
      toastStore.push(contentKind === "plugin" ? "Your plugin was published." : "Your mod was published.", "success");
      message = `${created.title} created as ${created.status}.`;
      await goto(contentKind === "plugin" ? `/plugins/${created.slug}` : `/skins/${created.slug}`);
    } catch (err) {
      error = err instanceof Error ? err.message : "Upload failed";
      isSubmitting = false;
    }
  }

  function addLink() {
    if (links.length >= 12) return;
    links = [...links, { label: "", url: "", kind: "external" }];
  }

  function removeLink(index: number) {
    links = links.filter((_, itemIndex) => itemIndex !== index);
    if (links.length === 0) links = [{ label: "", url: "", kind: "external" }];
  }

  function addVideo() {
    if (videos.length >= 4) return;
    videos = [...videos, { label: "", url: "" }];
  }

  function removeVideo(index: number) {
    videos = videos.filter((_, itemIndex) => itemIndex !== index);
    if (videos.length === 0) videos = [{ label: "", url: "" }];
  }

  function revokeImagePreviews() {
    for (const preview of imagePreviews) URL.revokeObjectURL(preview.url);
    imagePreviews = [];
  }

  function webpName(name: string) {
    return name.replace(/\.[^.]+$/, "") + ".webp";
  }

  async function convertImageToWebp(file: File) {
    if (file.type === "image/webp") {
      return { file, converted: false };
    }

    const bitmap = await createImageBitmap(file);
    const canvas = document.createElement("canvas");
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Unable to prepare image conversion");
    context.drawImage(bitmap, 0, 0);
    bitmap.close();

    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/webp", webpQuality));
    if (!blob) throw new Error(`Could not convert ${file.name} to WEBP`);

    return {
      file: new File([blob], webpName(file.name), { type: "image/webp", lastModified: Date.now() }),
      converted: true
    };
  }

  async function processImages(selectedImages: File[]) {
    revokeImagePreviews();
    convertedImages = [];
    imageStatus = "";

    if (selectedImages.length === 0) return;

    try {
      const validImages = selectedImages.filter((file) => file.type.startsWith("image/"));
      const processed = await Promise.all(validImages.map(convertImageToWebp));
      convertedImages = processed.map((item) => item.file);
      imagePreviews = processed.map((item) => ({
        name: item.file.name,
        size: item.file.size,
        url: URL.createObjectURL(item.file),
        converted: item.converted
      }));
      const convertedCount = processed.filter((item) => item.converted).length;
      imageStatus = `${processed.length} image${processed.length > 1 ? "s" : ""} ready${convertedCount ? `, ${convertedCount} converted to WEBP` : ""}.`;
    } catch (err) {
      convertedImages = [];
      error = err instanceof Error ? err.message : "Image conversion failed";
    }
  }

  async function handleImagesChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    await processImages(Array.from(input.files ?? []));
    if (!convertedImages.length) input.value = "";
  }

  async function handleImageDrop(event: DragEvent) {
    event.preventDefault();
    isDraggingImages = false;
    await processImages(Array.from(event.dataTransfer?.files ?? []));
  }

  function handleArchiveDrop(event: DragEvent) {
    event.preventDefault();
    isDraggingArchive = false;
    const expectedExtension = contentKind === "plugin" ? ".dll" : ".zip";
    const droppedFiles = Array.from(event.dataTransfer?.files ?? []).filter((file) => file.name.toLowerCase().endsWith(expectedExtension));
    const transfer = new DataTransfer();
    for (const file of droppedFiles) transfer.items.add(file);
    files = transfer.files;
  }

  function formatBytes(bytes: number) {
    if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function toneClass(tone: ChecklistTone) {
    if (tone === "green") return "border-emerald-400/25 bg-emerald-500/10 text-emerald-200";
    if (tone === "orange") return "border-amber-400/30 bg-amber-500/10 text-amber-200";
    return "border-red-400/30 bg-red-500/10 text-red-200";
  }

  function toneDotClass(tone: ChecklistTone) {
    if (tone === "green") return "bg-emerald-300";
    if (tone === "orange") return "bg-amber-300";
    return "bg-red-300";
  }

  function selectContentKind(next: "mod" | "plugin") {
    if (contentKind === next) return;
    contentKind = next;
    files = null;
  }

  onDestroy(revokeImagePreviews);
</script>

<form class="grid gap-5" on:submit|preventDefault={submitSkin}>
  <Card class="p-5">
    <div class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
      <div>
        <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Upload</p>
        <h1 class="mt-1 text-3xl font-black tracking-tight">New skin</h1>
        <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">Prepare the listing, check the live preview, then resolve every red item before submitting.</p>
      </div>
      <Button href="/" variant="ghost" size="sm">Cancel</Button>
    </div>

    <div class="mt-5 grid gap-2 rounded-lg border border-white/10 bg-background/45 p-1 sm:w-fit sm:grid-cols-2">
      <button
        class="rounded-md px-5 py-3 text-left text-sm font-black transition {contentKind === 'mod' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-white/10 hover:text-foreground'}"
        type="button"
        on:click={() => selectContentKind("mod")}
      >
        Mods
      </button>
      <button
        class="rounded-md px-5 py-3 text-left text-sm font-black transition {contentKind === 'plugin' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-white/10 hover:text-foreground'}"
        type="button"
        on:click={() => selectContentKind("plugin")}
      >
        Plugins
      </button>
    </div>
  </Card>

  <div class="grid items-start gap-5 xl:grid-cols-[minmax(0,1fr)_380px]">
    <div class="grid gap-5">
      <Card class="p-5">
        <div class="mb-5 flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <h2 class="text-lg font-bold">{contentKind === "plugin" ? "Plugin details" : "Mod details"}</h2>
            <p class="mt-1 text-sm text-muted-foreground">Use Markdown for headings, lists, links, bold text, and code blocks.</p>
          </div>
          <span class="rounded-full border border-white/10 bg-background/55 px-3 py-1 text-xs font-bold text-muted-foreground">{description.length}/4000</span>
        </div>
        <div class="grid gap-4 sm:grid-cols-2">
          <Label>
            Title
            <Input bind:value={title} placeholder="Gear Five recolor" />
          </Label>
          <Label>
            Version
            <Input bind:value={version} placeholder="1.0.0" />
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
          <Label class="sm:col-span-2">
            Character
            <CharacterCombobox {characters} bind:value={characterId} placeholder="Choose a character" valueKey="id" includeAll={false} />
          </Label>
          {/if}
          <Label class="sm:col-span-2">
            {contentKind === "plugin" ? "Short description" : "Description"}
            <textarea
              class="min-h-44 w-full rounded-md border border-white/12 bg-background/55 px-3 py-2 text-sm leading-6 text-foreground shadow-sm outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring"
              maxlength="4000"
              bind:value={description}
              placeholder={"## What changed?\n- New outfit colors\n- Tested on current Steam build\n\n[Creator page](https://example.com)"}
            ></textarea>
          </Label>
          {#if contentKind === "plugin"}
            <Label>
              Lua module name
              <Input bind:value={luaModuleName} placeholder="fx_director" />
            </Label>
            <Label class="sm:col-span-2">
              Source code URL
              <Input bind:value={sourceCodeUrl} placeholder="https://github.com/creator/plugin" />
            </Label>
          {/if}
        </div>
      </Card>

      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-lg font-bold">Media</h2>
          <p class="mt-1 text-sm text-muted-foreground">Add external videos and screenshots. Videos are embedded from the source, never hosted here.</p>
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
              <Label>
                Label
                <Input bind:value={video.label} placeholder="Gameplay preview" />
              </Label>
              <Label>
                URL
                <Input bind:value={video.url} placeholder="https://www.youtube.com/watch?v=..." />
              </Label>
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
        <div
          role="group"
          aria-label="Preview image dropzone"
          class="grid gap-4 rounded-lg border border-dashed p-4 transition {isDraggingImages ? 'border-primary bg-primary/10' : 'border-white/14 bg-background/45'}"
          on:dragover|preventDefault={() => (isDraggingImages = true)}
          on:dragleave={() => (isDraggingImages = false)}
          on:drop={handleImageDrop}
        >
          <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <p class="text-sm font-black">Preview images</p>
              <p class="mt-1 text-xs text-muted-foreground">Drop images here or choose files.</p>
            </div>
            <label class="inline-flex h-10 cursor-pointer items-center justify-center rounded-md border border-white/14 bg-background/70 px-4 text-sm font-bold text-foreground hover:bg-white/10">
              Choose images
              <input class="sr-only" type="file" accept="image/png,image/jpeg,image/webp,image/gif" multiple on:change={handleImagesChange} />
            </label>
          </div>
          {#if imageStatus}
            <p class="text-xs text-muted-foreground">{imageStatus}</p>
          {/if}
          {#if imagePreviews.length}
            <div class="grid gap-3 sm:grid-cols-2">
              {#each imagePreviews as preview}
                <div class="overflow-hidden rounded-lg border border-white/10 bg-background/55">
                  <div class="aspect-video bg-black/60">
                    <img class="h-full w-full object-cover" src={preview.url} alt={preview.name} />
                  </div>
                  <div class="flex items-center justify-between gap-3 px-3 py-2 text-xs">
                    <span class="min-w-0 truncate font-bold text-foreground">{preview.name}</span>
                    <span class="shrink-0 text-muted-foreground">{formatBytes(preview.size)}</span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </Card>

      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-lg font-bold">Credits</h2>
          <p class="mt-1 text-sm text-muted-foreground">Your own mods use your account profile. Creator credits can exist without a linked account.</p>
        </div>
        <div class="grid gap-4 sm:grid-cols-2">
          <Label>
            Ownership
            <select class="h-10 rounded-md border border-white/12 bg-background/55 px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring" bind:value={ownershipType}>
              <option value="own_work">My work</option>
              <option value="community_repost">Creator</option>
            </select>
          </Label>
          <Label>
            Tags
            <Input bind:value={tags} placeholder="wano, recolor, coat" />
          </Label>
          {#if contentKind === "mod"}
            <Label>
              Plugin dependencies
              <Input bind:value={pluginDependencies} placeholder="lua-core, costume-api" />
            </Label>
          {/if}
          {#if !isOwnWork}
            <Label>
              Creator name
              <Input bind:value={externalCreatorName} placeholder="Creator or team name" />
            </Label>
            <Label>
              Source URL
              <Input bind:value={externalCreatorUrl} placeholder="https://..." />
            </Label>
          {/if}
        </div>
      </Card>

      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-lg font-bold">Obtain</h2>
          <p class="mt-1 text-sm text-muted-foreground">{contentKind === "plugin" ? "Upload exactly one DLL. The launcher installs it into plugins/<plugin>/<dll>." : "Add a public obtain link, a hosted ZIP, or both."}</p>
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
                  <Label>
                    Label
                    <Input bind:value={link.label} placeholder="Patreon" />
                  </Label>
                  <Label>
                    URL
                    <Input bind:value={link.url} placeholder="https://..." />
                  </Label>
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
                <h3 class="text-base font-black">{contentKind === "plugin" ? "Hosted DLL" : "Hosted ZIP"}</h3>
                <p class="mt-1 text-xs leading-5 text-muted-foreground">{contentKind === "plugin" ? "Required for plugin uploads." : "Optional. Use when users should download the file directly from this hub."}</p>
              </div>
              <span class="w-fit rounded-full border border-white/12 bg-background/55 px-2.5 py-1 text-xs font-black text-muted-foreground">{archiveFiles.length} file{archiveFiles.length > 1 ? "s" : ""}</span>
            </div>

            <div
              role="group"
              aria-label={contentKind === "plugin" ? "DLL file dropzone" : "ZIP file dropzone"}
              class="grid gap-3 rounded-md border border-dashed p-4 transition sm:grid-cols-[1fr_auto] sm:items-center {isDraggingArchive ? 'border-primary bg-primary/10' : 'border-white/14 bg-background/35'}"
              on:dragover|preventDefault={() => (isDraggingArchive = true)}
              on:dragleave={() => (isDraggingArchive = false)}
              on:drop={handleArchiveDrop}
            >
              <div>
                <p class="text-sm font-black">{contentKind === "plugin" ? "Drop a DLL here" : "Drop ZIP files here"}</p>
                <p class="mt-1 text-xs leading-5 text-muted-foreground">{contentKind === "plugin" ? "DLL only. One file. 20 MB max." : "ZIP only. 20 MB max per file."}</p>
              </div>
              <input
                class="w-full rounded-md border border-white/12 bg-background/55 px-3 py-2 text-sm text-foreground file:mr-3 file:rounded-md file:border-0 file:bg-primary file:px-3 file:py-1.5 file:text-sm file:font-medium file:text-primary-foreground sm:w-72"
                bind:files={files}
                type="file"
                accept={contentKind === "plugin" ? ".dll" : ".zip,application/zip"}
                multiple={contentKind !== "plugin"}
              />
            </div>

            {#if archiveFiles.length}
              <div class="grid gap-2 text-xs sm:grid-cols-2">
                {#each archiveFiles as file}
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
            <p class="mt-1 text-sm text-muted-foreground">Check how this upload will read before it goes live.</p>
          </div>
          <div class="grid grid-cols-2 rounded-md border border-white/10 bg-background/45 p-1 text-sm font-bold">
            <button class="rounded px-3 py-1.5 {previewMode === 'card' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}" type="button" on:click={() => (previewMode = "card")}>Card</button>
            <button class="rounded px-3 py-1.5 {previewMode === 'page' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}" type="button" on:click={() => (previewMode = "page")}>Page</button>
          </div>
        </div>

        {#if previewMode === "card"}
          <article class="max-w-md overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.28)]">
            <div class="relative aspect-[16/11] overflow-hidden bg-muted">
              {#if primaryImage}
                <img class="h-full w-full object-cover" src={primaryImage.url} alt={titleValue || "Skin preview"} />
              {:else}
                <div class="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/.22),hsl(var(--accent)/.18))]"></div>
                <div class="absolute left-5 top-5 rounded-md border border-white/30 bg-white/12 px-4 py-3 text-4xl font-black text-white shadow-xl backdrop-blur">
                  {(selectedCharacter?.displayName ?? "SK").slice(0, 2).toUpperCase()}
                </div>
              {/if}
              <div class="absolute left-3 top-3 rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">
                {selectedCharacter?.isDlc ? "DLC" : "Base"}
              </div>
              <div class="absolute right-3 top-3 rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">
                {modTypeLabel(modType)}
              </div>
            </div>
            <div class="grid gap-4 p-4">
              <div>
                <p class="truncate text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">{selectedCharacter?.displayName ?? "Choose a character"}</p>
                <h3 class="mt-1 line-clamp-2 text-2xl font-black leading-tight">{titleValue || "Untitled skin"}</h3>
                <p class="mt-1 text-xs font-bold text-muted-foreground">v{version.trim() || "1.0.0"}</p>
              </div>
              <p class="line-clamp-2 min-h-11 text-sm leading-5 text-muted-foreground">{descriptionPreview || "No description yet."}</p>
              <div class="flex items-center justify-between gap-3 border-t border-white/10 pt-4 text-sm">
                <span class="min-w-0 truncate font-bold text-primary">{creatorName || "uncredited"}</span>
                <span class="inline-flex overflow-hidden rounded-md border border-white/14 bg-background/60 text-sm font-black">
                  <span class="border-r border-white/10 px-2.5 py-1.5 text-primary">▲</span>
                  <span class="min-w-9 px-2.5 py-1.5 text-center">0</span>
                </span>
              </div>
              {#if tagList.length}
                <div class="flex flex-wrap gap-2">
                  {#each tagList.slice(0, 3) as tag}
                    <span class="rounded-full border border-white/12 bg-background/60 px-2.5 py-1 text-xs font-bold text-muted-foreground">{tag}</span>
                  {/each}
                </div>
              {/if}
            </div>
          </article>
        {:else}
          <div class="grid gap-4">
            <div class="overflow-hidden rounded-lg border border-white/10 bg-card/70">
              {#if primaryImage}
                <div class="relative aspect-video overflow-hidden bg-black">
                  <img class="absolute inset-0 h-full w-full scale-110 object-cover opacity-30 blur-2xl" src={primaryImage.url} alt="" aria-hidden="true" />
                  <div class="absolute inset-0 bg-black/52"></div>
                  <img class="relative z-10 h-full w-full object-contain" src={primaryImage.url} alt={titleValue || "Skin preview"} />
                </div>
              {:else}
                <div class="grid aspect-video place-items-center bg-muted text-5xl font-black text-muted-foreground">{(selectedCharacter?.displayName ?? "SK").slice(0, 2).toUpperCase()}</div>
              {/if}
            </div>
            <div class="rounded-lg border border-white/10 bg-background/45 p-5">
              <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">{selectedCharacter?.displayName ?? "Choose a character"}</p>
              <div class="mt-2 flex flex-wrap items-end gap-3">
                <h3 class="text-3xl font-black tracking-tight">{titleValue || "Untitled skin"}</h3>
                <span class="rounded-md border border-white/12 bg-background/55 px-2 py-1 text-xs font-black text-muted-foreground">v{version.trim() || "1.0.0"}</span>
                <span class="rounded-md border border-primary/30 bg-primary/10 px-2 py-1 text-xs font-black text-primary">{modTypeLabel(modType)}</span>
              </div>
              {#if primaryVideo}
                <div class="mt-4 rounded-lg border border-white/10 bg-background/55 p-4">
                  <p class="text-sm font-black">{primaryVideo.label}</p>
                  <p class="mt-1 break-all text-xs text-primary">{primaryVideo.url}</p>
                </div>
              {/if}
              <div class="mt-4">
                <MarkdownContent value={descriptionValue} />
              </div>
              {#if tagList.length}
                <div class="mt-5 flex flex-wrap gap-2">
                  {#each tagList as tag}
                    <span class="rounded-full border border-white/12 bg-background/60 px-2.5 py-1 text-xs font-bold text-muted-foreground">{tag}</span>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </Card>
    </div>

    <aside class="grid content-start gap-5 xl:sticky xl:top-24">
      <Card class="p-5">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h2 class="text-lg font-bold">Readiness</h2>
            <p class="mt-1 text-sm text-muted-foreground">Red blocks submit. Orange is optional advice.</p>
          </div>
          <div class="rounded-md border border-white/10 bg-background/55 px-3 py-2 text-right">
            <p class="text-xs text-muted-foreground">Issues</p>
            <p class="text-xl font-black">{blockerCount}</p>
          </div>
        </div>

        <div class="mt-5 grid gap-2">
          {#each checklist as item}
            <div class="rounded-lg border px-3 py-2 {toneClass(item.tone)}">
              <div class="flex items-start gap-3">
                <span class="mt-1.5 h-2.5 w-2.5 shrink-0 rounded-full {toneDotClass(item.tone)}"></span>
                <div class="min-w-0">
                  <p class="text-sm font-black">{item.label}</p>
                  <p class="mt-0.5 text-xs leading-5 opacity-85">{item.description}</p>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </Card>

      <Card class="p-5">
        <Button type="submit" class="w-full" disabled={!canSubmit}>{isSubmitting ? "Creating..." : "Create skin"}</Button>
        <p class="mt-3 text-center text-xs text-muted-foreground">
          {#if blockerCount}
            Resolve {blockerCount} red item{blockerCount > 1 ? "s" : ""} before submitting.
          {:else if suggestionCount}
            Ready to submit. {suggestionCount} optional improvement{suggestionCount > 1 ? "s" : ""} remain.
          {:else}
            Ready to submit.
          {/if}
        </p>
      </Card>
    </aside>
  </div>

  {#if message}
    <div class="rounded-xl border border-emerald-400/30 bg-emerald-500/15 px-4 py-3 text-sm text-emerald-100">{message}</div>
  {/if}
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100">{error}</div>
  {/if}
</form>
