<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import VideoPlayer from 'svelte-video-player';

  let show = ""
  let src = ""
  async function getVideoPath(show: string) {
   //const file: string = await invoke("get_video_path", { show: show })
   let file = 'F:/Community/0411.mp4'
   return convertFileSrc(file)
  }

  getVideoPath(show).then(res => {src = res})
</script>

<main class="container">
 <VideoPlayer
 width="1920"
 height="1080"
 source="{src}"
 />
</main>

<style>
 main {
  max-height: 90vh;
 }
</style>
