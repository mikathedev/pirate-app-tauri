<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { open } from '@tauri-apps/plugin-dialog';
  import VideoPlayer from 'svelte-video-player';


  let show = ""
  let src = ""
  let dialog: HTMLDialogElement

  async function getVideoPath(show: string) {
   //const file: string = await invoke("get_video_path", { show: show })
   let file = 'F:/Community/0411.mp4'
   return convertFileSrc(file)
  }

  getVideoPath(show).then(res => {src = res})
</script>

<nav>
 <div data-tauri-drag-region></div>
 <button onclick={() => getVideoPath(show).then(res => {src = res})}>Get Video</button>
 <button class="open-modal" onclick={() => dialog.showModal()}>Open Modal</button>
</nav>
<main class="container">
 <VideoPlayer
 width="1920"
 height="1000"
 source="{src}"
 />
</main>




<dialog bind:this={dialog}>
 <p>This is a native modal!</p>
 <button onclick={() => dialog.close()}>Close</button>
</dialog>

<style>
 main {
  max-width: 97vw;
 }

 nav {
  display: flex;
  height: 4vh;
 }

 nav div {
  flex: 1;
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
</style>
