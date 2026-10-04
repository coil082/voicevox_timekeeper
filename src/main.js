const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

window.addEventListener("DOMContentLoaded", async() => {
  console.log("load");
  const setting= document.getElementById("setting");
  const reading_textarea = document.getElementById("reading-text");
  const time_duration = document.getElementById("time-duration");
  const duration_unit = document.getElementById("duration-unit");
  setting.addEventListener("submit",(e)=>{
    e.preventDefault();
    console.log("unnko")
    invoke("setting",{
      readText:reading_textarea.value,
      timeDuration:parseInt(time_duration.value),
      durationUnit:duration_unit.value
    })
  });
  await listen("1hour", () => {
    console.log("1hour event received");

  });
});
