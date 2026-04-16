<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { open } from '@tauri-apps/plugin-dialog';
  import VideoPlayer from 'svelte-video-player';


  let show = ""
  let src = $state("")
  let dialog: HTMLDialogElement

  let playing = $state(false);

  // CHANGED: svelte action to track video state
  function trackPlaying(node: HTMLElement) {
   const video = node.querySelector('video');
   if (!video) return {};
   const onPlay = () => playing = true;
   const onPause = () => playing = false;
   video.addEventListener('play', onPlay);
   video.addEventListener('pause', onPause);
   return {
    destroy() {
     video.removeEventListener('play', onPlay);
     video.removeEventListener('pause', onPause);
    }
   };
  }

  async function getVideoPath(show: string) {
   //const file: string = await invoke("get_video_path", { show: show })
   let file = 'F:/Community/0411.mp4'
   src = convertFileSrc(file)
  }
  async function getVideoPath2(show: string) {
   //const file: string = await invoke("get_video_path", { show: show })
   let file = 'F:/Community/0410.mp4'
   src = convertFileSrc(file)
  }

  getVideoPath(show)
</script>

<nav>
 <button onclick={() => getVideoPath(show)}>Get Video</button>
 <button class="open-modal" onclick={() => dialog.showModal()}>Open Modal</button>
 <div data-tauri-drag-region></div>
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
