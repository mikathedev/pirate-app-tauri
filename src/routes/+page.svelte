<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import {getCurrentWindow} from "@tauri-apps/api/window";
  import { open } from '@tauri-apps/plugin-dialog';
  import VideoPlayer from 'svelte-video-player';


  let show = $state("Community")
  let src = $state("")
  let dialog: HTMLDialogElement
  let options = $state([])
  let playing = $state(false);

  function closeWindow() {
     const window = getCurrentWindow();
     window.close();
  }
  async function get_options() {
   options = await invoke("get_options")
   console.log(options)
  }
  // CHANGED: svelte action to track video state
  function trackPlaying(node: HTMLElement) {
   const video = node.querySelector('video');
   if (!video) return {};
   const onPlay = () => { playing = true };
   const onPause = () => { playing = false; }
   video.addEventListener('play', onPlay);
   video.addEventListener('pause', onPause);
   return {
    destroy() {
     video.removeEventListener('play', onPlay);
     video.removeEventListener('pause', onPause);
    }
   };
  }

  function downloadFile() {
   invoke("download", { show: show }).then((res) => {console.log(res)})
  }
  async function getVideoPath(show: string) {
   const file: string = await invoke("get_video_path", { show: show })
   src = convertFileSrc(file)
  }
  async function getVideoPath2(show: string) {
   //const file: string = await invoke("get_video_path", { show: show })
   let file = 'F:/Community/0410.mp4'
   src = convertFileSrc(file)
  }

  getVideoPath(show)
  get_options()
</script>

<nav>
 <select bind:value={show}>
    {#each options as show}
     <option value={show}>{show}</option>
    {/each}

 </select>
 <button onclick={() => getVideoPath(show)}>Get Video</button>
 <button onclick={() => downloadFile()}>run download</button>
 <button class="open-modal" onclick={() => dialog.showModal()}>Open Modal</button>
 <div data-tauri-drag-region></div>
 <button onclick={closeWindow}>
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
   <path
           d="M18 6L6 18M6 6L18 18"
           stroke="currentColor"
           stroke-width="2"
           stroke-linecap="round"
           stroke-linejoin="round"
   />
  </svg>
 </button>
</nav>
<main class="container">
 <div class="video-player" use:trackPlaying>
 {#key src}
  <VideoPlayer
  width="1920"
  height="1080"
  source={src}
  autoplay={true}
  />
 {/key}
 </div>
</main>

<dialog bind:this={dialog}>
 <p>This is a native modal!</p>
 <button onclick={() => getVideoPath2(show)}>Get Video2</button>
 <button onclick={() => dialog.close()}>Close</button>
</dialog>

<style>
 main {
  display: flex;
  justify-content: center;
 }

 nav {
  display: flex;
  min-height: 2rem;
  height: 2.5vh;
  margin-bottom: 1rem;
 }

 nav div {
  flex: 1;
 }

 nav div:hover {
  cursor: pointer;
 }

 button {
  background-color: #FF6331;
  border: none;
  color: white;
  padding: 7px 16px;
  margin: 0 2px 0 2px;
  text-align: center;
  text-decoration: none;
  display: inline-block;
  font-size: 16px;
  border-radius: 10px;
 }

 .video-player {
  width: 100%;
  max-width: calc(95vh * (16 / 9));
 }



</style>
