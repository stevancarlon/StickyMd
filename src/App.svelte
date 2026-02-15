<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { renderMarkdown } from "./lib/markdown";
  import { loadNote, saveNote, type NoteData } from "./lib/storage";

  type Mode = "edit" | "preview";

  let mode: Mode = $state("edit");
  let content = $state("");
  let color = $state("yellow");
  let pinned = $state(true);
  let opacity = $state(1.0);
  let borderRadius = $state(0);
  let editorEl: HTMLTextAreaElement | undefined = $state();

  const colors: Record<string, { bg: string; border: string; text: string }> = {
    yellow: { bg: "#fff9c4", border: "#f9e54a", text: "#5d4e00" },
    pink: { bg: "#f8bbd0", border: "#ec407a", text: "#4a0e23" },
    green: { bg: "#c8e6c9", border: "#66bb6a", text: "#1b3a1d" },
    blue: { bg: "#bbdefb", border: "#42a5f5", text: "#0d2b4a" },
    white: { bg: "#f5f5f5", border: "#bdbdbd", text: "#212121" },
    dark: { bg: "#1e1e1e", border: "#333333", text: "#d4d4d4" },
    "dark-blue": { bg: "#1a1b2e", border: "#2d2f54", text: "#a9b1d6" },
    "dark-green": { bg: "#1a2e1a", border: "#2d542d", text: "#a9d6a9" },
    "dark-purple": { bg: "#2a1a2e", border: "#4a2d54", text: "#c9a9d6" },
  };

  let html = $derived(renderMarkdown(content));
  let currentColors = $derived(colors[color] || colors.yellow);
  let isDark = $derived(color.startsWith("dark"));

  onMount(async () => {
    const note = await loadNote();
    content = note.content;
    color = note.color;
    pinned = note.pinned;
    opacity = note.opacity ?? 1.0;
    borderRadius = note.border_radius ?? 0;
    await invoke("set_pinned", { pinned });
    await invoke("set_opacity", { opacity });
  });

  function noteData() {
    return { content, color, pinned, opacity, border_radius: borderRadius };
  }

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    content = target.value;
    saveNote(noteData());
  }

  async function togglePin() {
    pinned = !pinned;
    await invoke("set_pinned", { pinned });
    saveNote(noteData());
  }

  async function hideWindow() {
    await getCurrentWindow().hide();
  }

  // --- Drag implementation ---
  // Use native startDragging() which works with non-transparent
  // frameless windows on both X11 and Wayland.
  async function onDragMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest("button")) return;
    e.preventDefault();
    e.stopPropagation();
    await getCurrentWindow().startDragging();
  }

  function setColor(c: string) {
    color = c;
    saveNote(noteData());
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === "e") {
      e.preventDefault();
      mode = "edit";
      setTimeout(() => editorEl?.focus(), 10);
    }
    if (e.ctrlKey && e.key === "p") {
      e.preventDefault();
      mode = "preview";
    }
  }

  let showColorPicker = $state(false);
  let showSettings = $state(false);

  async function handleOpacityChange(e: Event) {
    opacity = parseFloat((e.target as HTMLInputElement).value);
    await invoke("set_opacity", { opacity });
    saveNote(noteData());
  }

  function handleRadiusChange(e: Event) {
    borderRadius = parseInt((e.target as HTMLInputElement).value, 10);
    saveNote(noteData());
  }

  function handlePreviewClick(e: MouseEvent) {
    // Walk up from the click target to find a .md-checkbox span
    const target = (e.target as HTMLElement).closest(".md-checkbox") as HTMLElement | null;
    if (!target) return;

    const clickedIndex = parseInt(target.dataset.checkboxIndex ?? "-1", 10);
    if (clickedIndex < 0) return;

    // Find the matching checkbox in the raw markdown and toggle it
    const checkboxRegex = /- \[([ xX])\]/g;
    let match;
    let idx = 0;
    while ((match = checkboxRegex.exec(content)) !== null) {
      if (idx === clickedIndex) {
        const isChecked = match[1] !== " ";
        const replacement = isChecked ? "- [ ]" : "- [x]";
        content = content.slice(0, match.index) + replacement + content.slice(match.index + match[0].length);
        saveNote(noteData());
        break;
      }
      idx++;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="note"
  style="background-color: {currentColors.bg}; border-color: {currentColors.border}; color: {currentColors.text}; border-radius: {borderRadius}px;"
>
  <!-- Title Bar -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="titlebar" onmousedown={onDragMouseDown}>
    <span class="title">Sticky MD</span>
    <div class="controls">
      <button
        class="btn"
        class:active={pinned}
        onclick={togglePin}
        title={pinned ? "Unpin" : "Pin on top"}
      >
        {#if pinned}
          <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
            <path d="M9.828.722a.5.5 0 0 1 .354.146l4.95 4.95a.5.5 0 0 1-.707.708l-.812-.813-3.04 3.04a4 4 0 0 1 .716 1.46l.293 1.17a.5.5 0 0 1-.82.45l-2.47-2.47-4.585 4.585a.5.5 0 0 1-.708-.708l4.586-4.585-2.47-2.47a.5.5 0 0 1 .45-.82l1.17.293a4 4 0 0 1 1.46.716l3.04-3.04-.813-.812a.5.5 0 0 1 .146-.354z"/>
          </svg>
        {:else}
          <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
            <path d="M9.828.722a.5.5 0 0 1 .354.146l4.95 4.95a.5.5 0 0 1-.707.708l-.812-.813-3.04 3.04a4 4 0 0 1 .716 1.46l.293 1.17a.5.5 0 0 1-.82.45l-2.47-2.47-4.585 4.585a.5.5 0 0 1-.708-.708l4.586-4.585-2.47-2.47a.5.5 0 0 1 .45-.82l1.17.293a4 4 0 0 1 1.46.716l3.04-3.04-.813-.812a.5.5 0 0 1 .146-.354z" opacity="0.4"/>
          </svg>
        {/if}
      </button>
      <button class="btn" onclick={hideWindow} title="Hide to tray">
        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
          <path d="M3 8a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9A.5.5 0 0 1 3 8z"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Tab Bar -->
  <div class="tabs">
    <div class="tab-group">
      <button class="tab" class:active={mode === "edit"} onclick={() => { mode = "edit"; setTimeout(() => editorEl?.focus(), 10); }}>
        Edit
      </button>
      <button class="tab" class:active={mode === "preview"} onclick={() => (mode = "preview")}>
        Preview
      </button>
    </div>
    <div class="tab-actions">
      <div class="color-picker-wrapper">
        <button class="btn" onclick={() => { showColorPicker = !showColorPicker; showSettings = false; }} title="Change color">
          <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
            <circle cx="8" cy="8" r="6" fill={currentColors.border}/>
          </svg>
        </button>
        {#if showColorPicker}
          <div class="color-picker" style="background: {isDark ? '#2a2a2a' : 'white'};">
            {#each Object.entries(colors) as [name, c]}
              <button
                class="color-swatch"
                class:selected={color === name}
                style="background-color: {c.bg}; border-color: {c.border};"
                onclick={() => { setColor(name); showColorPicker = false; }}
                title={name}
              ></button>
            {/each}
          </div>
        {/if}
      </div>
      <div class="settings-wrapper">
        <button class="btn" onclick={() => { showSettings = !showSettings; showColorPicker = false; }} title="Settings">
          <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
            <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
            <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.115l.094-.319z"/>
          </svg>
        </button>
        {#if showSettings}
          <div class="settings-panel" style="background: {isDark ? '#2a2a2a' : 'white'}; color: {isDark ? '#d4d4d4' : '#333'};">
            <label class="setting-row">
              <span class="setting-label">Opacity</span>
              <input type="range" min="0.2" max="1" step="0.05" value={opacity} oninput={handleOpacityChange} />
              <span class="setting-value">{Math.round(opacity * 100)}%</span>
            </label>
            <label class="setting-row">
              <span class="setting-label">Roundness</span>
              <input type="range" min="0" max="24" step="1" value={borderRadius} oninput={handleRadiusChange} />
              <span class="setting-value">{borderRadius}px</span>
            </label>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Content -->
  <div class="content">
    {#if mode === "edit"}
      <textarea
        bind:this={editorEl}
        class="editor"
        value={content}
        oninput={handleInput}
        placeholder="Write markdown here..."
        spellcheck="true"
      ></textarea>
    {:else}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="preview" onclick={handlePreviewClick}>
        {@html html}
      </div>
    {/if}
  </div>
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    font-family: -apple-system, "Segoe UI", "Ubuntu", "Cantarell", sans-serif;
  }

  .note {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    border: 1.5px solid;
    clip-path: inset(0 round inherit);
  }

  .titlebar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 34px;
    padding: 0 10px;
    user-select: none;
    cursor: grab;
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
    flex-shrink: 0;
  }

  .title {
    font-size: 11px;
    font-weight: 700;
    opacity: 0.5;
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .controls {
    display: flex;
    gap: 2px;
  }

  .btn {
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: inherit;
    opacity: 0.6;
    transition: all 0.15s;
  }

  .btn:hover {
    background: rgba(0, 0, 0, 0.1);
    opacity: 1;
  }

  .btn.active {
    opacity: 1;
  }

  .tabs {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 6px;
    height: 30px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
    flex-shrink: 0;
  }

  .tab-group {
    display: flex;
    gap: 0;
  }

  .tab {
    padding: 4px 12px;
    border: none;
    background: transparent;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    opacity: 0.4;
    border-bottom: 2px solid transparent;
    color: inherit;
    transition: all 0.15s;
  }

  .tab:hover {
    opacity: 0.7;
  }

  .tab.active {
    opacity: 1;
    border-bottom-color: currentColor;
  }

  .tab-actions {
    display: flex;
    align-items: center;
  }

  .color-picker-wrapper,
  .settings-wrapper {
    position: relative;
  }

  .color-picker {
    position: absolute;
    top: 100%;
    right: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 6px;
    background: white;
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    z-index: 100;
    max-width: 140px;
  }

  .color-swatch {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid;
    cursor: pointer;
    transition: transform 0.15s;
  }

  .color-swatch:hover {
    transform: scale(1.2);
  }

  .color-swatch.selected {
    transform: scale(1.3);
  }

  .settings-panel {
    position: absolute;
    top: 100%;
    right: 0;
    padding: 10px 12px;
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    z-index: 100;
    min-width: 200px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    cursor: default;
  }

  .setting-label {
    width: 60px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .setting-row input[type="range"] {
    flex: 1;
    height: 4px;
    cursor: pointer;
    accent-color: currentColor;
  }

  .setting-value {
    width: 32px;
    text-align: right;
    font-size: 10px;
    opacity: 0.7;
  }

  .content {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .editor {
    width: 100%;
    height: 100%;
    border: none;
    outline: none;
    resize: none;
    padding: 12px;
    font-family: "JetBrains Mono", "Fira Code", "Ubuntu Mono", "Consolas",
      monospace;
    font-size: 13px;
    line-height: 1.6;
    background: transparent;
    color: inherit;
  }

  .preview {
    padding: 12px;
    font-size: 13px;
    line-height: 1.6;
    overflow-y: auto;
    height: 100%;
  }

  .preview :global(h1) {
    font-size: 1.5em;
    margin: 0.3em 0;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    padding-bottom: 0.2em;
  }
  .preview :global(h2) {
    font-size: 1.3em;
    margin: 0.3em 0;
  }
  .preview :global(h3) {
    font-size: 1.1em;
    margin: 0.2em 0;
  }
  .preview :global(p) {
    margin: 0.4em 0;
  }
  .preview :global(code) {
    background: rgba(128, 128, 128, 0.15);
    padding: 1px 5px;
    border-radius: 4px;
    font-size: 0.9em;
    font-family: "JetBrains Mono", "Fira Code", "Ubuntu Mono", monospace;
  }
  .preview :global(pre) {
    background: rgba(128, 128, 128, 0.1);
    border-radius: 6px;
    padding: 8px 10px;
    overflow-x: auto;
    margin: 0.4em 0;
  }
  .preview :global(pre code) {
    background: none;
    padding: 0;
  }
  .preview :global(ul),
  .preview :global(ol) {
    padding-left: 20px;
    margin: 0.3em 0;
  }
  .preview :global(li) {
    margin: 0.15em 0;
  }
  .preview :global(blockquote) {
    border-left: 3px solid rgba(0, 0, 0, 0.2);
    padding-left: 10px;
    margin: 0.4em 0;
    opacity: 0.8;
  }
  .preview :global(a) {
    color: inherit;
    text-decoration: underline;
    opacity: 0.85;
  }
  .preview :global(a:hover) {
    text-decoration: underline;
  }
  .preview :global(hr) {
    border: none;
    border-top: 1px solid rgba(0, 0, 0, 0.1);
    margin: 0.5em 0;
  }
  .preview :global(.task-list-item) {
    list-style: none;
    margin-left: -20px;
  }
  .preview :global(.md-checkbox) {
    cursor: pointer;
    margin-right: 4px;
    font-size: 1.15em;
    vertical-align: middle;
    user-select: none;
    opacity: 0.8;
    transition: opacity 0.1s;
  }
  .preview :global(.md-checkbox:hover) {
    opacity: 1;
  }
  .preview :global(table) {
    border-collapse: collapse;
    margin: 0.4em 0;
    font-size: 0.9em;
  }
  .preview :global(th),
  .preview :global(td) {
    border: 1px solid rgba(0, 0, 0, 0.15);
    padding: 4px 8px;
  }
  .preview :global(th) {
    font-weight: 600;
    background: rgba(0, 0, 0, 0.04);
  }
</style>
