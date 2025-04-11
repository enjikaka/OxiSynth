function create_btn(label, noteOnCb, noteOffCb) {
  const btn = document.createElement("button");
  btn.classList.add("btn");
  btn.classList.add("waves-effect");
  btn.id = label;
  btn.innerText = label;

  btn.addEventListener("mousedown", noteOnCb);
  btn.addEventListener("mouseup", noteOffCb);

  return btn;
}

import("./pkg")
  .catch(console.error)
  .then((rust_module) => {
    document.getElementById("start").addEventListener("click", async () => {
      const soundfontFile = document.getElementById("soundfont-file").files[0];
      const fileBuffer = await soundfontFile.arrayBuffer();
      const uint8Array = new Uint8Array(fileBuffer);
      
      let handle = rust_module.loadSoundFont(uint8Array);

      const notLoaded = document.getElementById("not-loaded");
      const loaded = document.getElementById("loaded");
      const noteBtns = document.getElementById("note-btns");

      const labels = [
        "C",
        "C#",
        "D",
        "D#",
        "E",
        "F",
        "F#",
        "G",
        "G#",
        "A",
        "A#",
        "H",
      ];

      for (let id = 0; id < 12; id++) {
        const btn = create_btn(labels[id], () => {
          rust_module.noteOn(handle, 60 + id);
        }, () => {
          rust_module.noteOff(handle, 60 + id);
        });
        noteBtns.appendChild(btn);
      }

      notLoaded.style.display = "none";
      loaded.style.display = "block";
    });
  });
