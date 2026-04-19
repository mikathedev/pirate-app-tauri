<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import {getCurrentWindow} from "@tauri-apps/api/window";
  import { open } from '@tauri-apps/plugin-dialog';
  import {onMount} from "svelte";


  let show = $state("Community")
  let src = $state("")
  let dialog: HTMLDialogElement
  let options = $state([])

  function closeWindow() {
     const window = getCurrentWindow();
     window.close();
  }
  async function get_options() {
   options = await invoke("get_options")
   console.log(options)
  }

  function downloadFile() {
   invoke("download", { show: show }).then((res) => {console.log(res)})
  }
  async function getVideoPath(show: string) {
   const file: string = await invoke("get_video_path", { show: show })
   src = convertFileSrc(file)
  }

  onMount((): void =>{
   get_options()
   invoke("scrape", { show: show }).then((res) => {console.log(res)})
  })

</script>

<div class="navwrap">
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
 <button aria-label="close" onclick={closeWindow}>
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
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
</div>

{#key src}
 <video src={src} controls style="width:100%" onended={() => console.log('video ended!')}
 ><track kind="captions" src=""></video>
{/key}


<dialog bind:this={dialog}>
 <p>This is a native modal!</p>
 <button onclick={() => getVideoPath(show)}>Get Video2</button>
 <button onclick={() => dialog.close()}>Close</button>
</dialog>

<style>
 nav {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  max-width: calc(100vw - 20px);
  z-index: 10;
  display: flex;
  padding: 10px;
  background-color: #1a1a1a;
  transform: translateY(-100%);
  transition: all 0.2s ease-in-out;
 }
 .navwrap {
  position: fixed;
  top: 0;
  width: 100%;
  height: 80px;
 }

 .navwrap:hover nav {
  transform: translateY(0);
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

 /* Styling for the dropdown / select menu */
 select {
  background-color: #FF6331;
  border: 2px solid #FF6331; /* Matching your button color */
  color: white;
  padding: 6px 12px;
  font-size: 16px;
  border-radius: 10px; /* Consistent with your buttons */
  cursor: pointer;
  outline: none;
  transition: all 0.2s ease-in-out;
 }

 select:hover {
  background-color: #fb6b3d; /* Very light tint of your accent color */
 }

 select:focus {
  box-shadow: 0 0 5px rgba(255, 99, 49, 0.5);
 }

 option {
  background: #1a1a1a;
  color: white;
 }


 /* Styling for the video container */
 video {
  flex: 1;
  width: auto;
  height: auto;
  min-height: 98vh;
  max-width: 100%;
  display: block;
  margin: 0 auto; /* Centers the video */
  border-radius: 15px; /* Slightly more rounded than buttons for large elements */
 }
</style>
