<script lang="ts">
 import {convertFileSrc, invoke} from "@tauri-apps/api/core";
 import {getCurrentWindow} from "@tauri-apps/api/window";
 import {onDestroy, onMount} from "svelte";
 import {open} from "@tauri-apps/plugin-dialog";
 import {listen} from "@tauri-apps/api/event";


 let show = $state("")
  let src = $state("")
  let dialog: HTMLDialogElement
  let options = $state([])
  let output: unknown = $state("")
  let downloaded: number = $state(0)
 let videoele: HTMLVideoElement

  function closeWindow() {
     const window = getCurrentWindow();
     window.close();
  }

  let Nopacity = $state(0)
  let Ntitle = $state("")
  let Nmessage = $state("")
  function notify(title: string, message: string) {
   Ntitle = title
   Nmessage = message
   Nopacity = 1
   setTimeout(() => {
    Nopacity = 0
   }, 3000)
  }

  //add show variables
 let urlIn = $state("")
 let nameIn = $state("")
 let pathIn = $state("")

 async function openPathF() {
  const path = await open({
   directory: true,
   multiple: false
  })
  if (path != null) {
   pathIn = path
  }
 }

 async function addShow() {
  if (!urlIn.includes("https://")) {
   urlIn = "https://" + urlIn
  }
  dialog.close()
  const status = await invoke("add_show", {name: nameIn, path: pathIn, url: urlIn})
  if (status == true) {
   notify("Add Show Update", "success adding show")
  } else if (typeof status == "string") { notify("Add Show Update", "error adding show: " + status) }
  return status
 }
  async function get_options() {
   options = await invoke("get_options")
   show = options[0]
   console.log(options)
  }

  function downloadFile() {
   invoke("download", { showstr: show })
  }
  async function getVideoPath() {
   const file: string = await invoke("get_video_path", { show: show })
   src = convertFileSrc(file)
   videoele.load()
   await videoele.play()
  }

  let unlisteners: (() => void)[] = []

  onMount(async () =>{
    await get_options()
    await invoke("do_i_download", {show: show})
    await getVideoPath()
    unlisteners.push(await listen("BE", (event) => {
     output = event.payload
    }))
    unlisteners.push(await listen("download", (event) => {
     downloaded = event.payload as number
    }))

    unlisteners.push(await listen("NextEpisode", (event) => {
     src = convertFileSrc(event.payload as string)
    }))
   unlisteners.push(await listen("downloadFinished", event => {
    notify("Download Progress", event.payload + "%")
    downloaded = 0
    invoke("do_i_download", {show: show}).then((res) => {
     if (res == true) {
      downloadFile()
     }})
   }))

  })
 onDestroy(() => {
  unlisteners.forEach(unlisten => unlisten())
 })

  function openDialog() {
   dialog.showModal()
   document.body.style.overflow = 'hidden'
  }

  function closeDialog() {
   dialog.close()
   document.body.style.overflow = '' // restore
  }
</script>

<div class="navwrap">
<nav>
 <select onchange={() => getVideoPath()} bind:value={show}>
    {#each options as show}
     <option value={show}>{show}</option>
    {/each}
 </select>
 <button onclick={() => invoke("ended", {show: show})}>Next Episode</button>
 <button class="open-modal" onclick={() => {openDialog()}}>Add Show</button>
 <progress max="100" value="{downloaded}"></progress>
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
 <video bind:this={videoele} src={src} controls style="width:100%" onended={() => {invoke("ended", {show: show})}}
 ><track kind="captions" src=""></video>
{/key}
<div class="notification" style="opacity: {Nopacity};"><h3>{Ntitle}</h3><p>{Nmessage}</p></div>

<dialog bind:this={dialog}>
 <form onsubmit={addShow}>
  <h1>Add Show</h1>
  <section>
   <button type="button" onclick={() => invoke("scrape", { show: show, first: true }).then((res) => {console.log(res)})}>scrape</button>
   <div><label for="#name">Show Name:  </label> <input required bind:value={nameIn} type="text" id="name" name="name" /></div>
   <div><label for="#path">Path:  </label> <input required bind:value={pathIn} type="text" id="path" name="path" /> <button tabindex="-1" onclick={openPathF} type="button">Open</button></div>
   <div><label for="#url">Url:  </label> <input required bind:value={urlIn} type="url" id="url" name="url" /></div>
   <button type="submit">Add</button>
  </section>
 </form>
 <button type="button" id="close" onclick={() => {closeDialog()}}>Close</button>
 <button onclick={() => invoke("ended", {show: show})}>ended</button>
</dialog>

<style>
 @import url('https://fonts.googleapis.com/css2?family=Roboto:ital,wght@0,100..900;1,100..900&display=swap');

 * {
  font-family: "Roboto", sans-serif;
  font-optical-sizing: auto;
  font-weight: normal;
  font-style: normal;
  font-variation-settings:
          "wdth" 100;
 }

 :global(html, body) {
  overflow: hidden;
  height: 100%;
  margin: 0;
  padding: 0;
 }
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

 h3 {
  padding: 10px;
  margin: 0;
  color: white;
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

 .notification {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background-color: #FF6331;
  color: white;
  padding: 7px;
  border-radius: 5px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  z-index: 1000;
  transition: opacity 0.1s ease-in-out;
  text-align: right;
 }

.notification h3 {
  margin: 0;
  padding: 0;
}

.notification p {
 margin: 0;
 padding: 0;
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

 progress {
  width: 10vw;
  height: 40px;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  border: none;

 }

 progress::-webkit-progress-value {
  background-color: #fb6b3d;
  border-radius: 10px;
 }

 progress::-webkit-progress-bar {
  background-color: #151414;
  border-radius: 10px;
 }

 /* Styling for the video container */
 video {
  width: 100%;
  height: 100vh; /* CHANGED: fixed height instead of min-height */
  max-width: 100%;
  display: block;
  margin: 0 auto;
  object-fit: contain; /* CHANGED: keeps aspect ratio without overflow */
 }

 /*  dialog  */

 dialog {
  background-color: #1a1a1a;
  color: white;
  border: 2px solid #FF6331;
  border-radius: 15px;
  padding: 20px;
  width: 80vw;
  max-width: 90%;
  height: 80vh;
  overflow: hidden; /* CHANGED: prevent scrolling */
 }

 dialog[open] {
  display: flex;
  flex-direction: column;
  justify-content: center;
 }

 form {
  display: flex;
  flex-direction: column;
  gap: 20px;
  width: 100%;
  justify-content: center;
  align-items: center;
 }

 form section div {
  align-content: center;
  width: 80%;
  gap: 10px;
  height: 30px;
  display: flex;
  align-items: flex-start;
  justify-content: flex-start;
 }
 form section {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 100%;
  justify-content: center;
  align-items: center;
 }

 form section div button {
  max-width: 30%;
  margin-left: 20px;
 }

form section label {
 margin: auto 20px auto 0;
 white-space: nowrap;
}

form section input {
 background-color: #131313;
 border: 2px solid #FF6331;
 color: white;
 padding: 6px 12px;
 font-size: 16px;
 border-radius: 10px;
 outline: none;
 transition: all 0.2s ease-in-out;
}

 form section button {
  max-width: 40%;
  width: 100%;
 }

 #close {
  margin-top: auto; /* now works correctly */
  margin-bottom: 2rem;
 }

</style>