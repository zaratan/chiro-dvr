"use strict";
const VerdictsBar = ({ root, store, say }) => {
  const counts = root.querySelector(".counts");
  const state = root.querySelector(".state");
  const readFrom = document.querySelector(".bar-info .read-from");
  const notice = document.querySelector(".bar-info .bar-notice");
  const draft = root.querySelector(".draft");
  const steps = root.querySelector(".steps");
  const folder = root.dataset.folder;
  let downloadedAt = "";
  let lastError = "";

  const update = () => {
    const totals = store.counts();
    counts.textContent = say("counts", {
      yes: totals.yes,
      no: totals.no,
      doubt: totals.doubt,
      todo: totals.todo,
    });
    const pending = store.pending();
    if (pending > 0) {
      downloadedAt = "";
      state.textContent = say("pending", { count: pending });
    } else if (downloadedAt) {
      state.textContent = say("downloaded", { time: downloadedAt });
    } else {
      state.textContent = say("current");
    }
    state.classList.toggle("pending", pending > 0 || Boolean(downloadedAt));
    const absent = store.absentCount();
    const lines = [lastError, absent > 0 ? say("absent", { count: absent }) : ""].filter(Boolean);
    notice.textContent = lines.join(" ");
    notice.hidden = lines.length === 0;
  };

  const showDownloaded = (time) => {
    downloadedAt = time;
    steps.querySelector(".step-one").textContent = say("step-one", { folder });
    steps.querySelector(".step-two").textContent = say("step-two", { folder });
    update();
  };

  steps.querySelector(".understood").addEventListener("click", () => {
    steps.hidden = true;
    window.dispatchEvent(new Event("resize"));
  });

  return {
    update,
    downloaded: (time) => {
      showDownloaded(time);
      steps.hidden = false;
      steps.querySelector(".steps-heading").focus();
      window.dispatchEvent(new Event("resize"));
    },
    remembered: showDownloaded,
    readFrom: ({ name, date, candidates }) => {
      readFrom.textContent = say("read-from", { name, date, count: candidates });
      readFrom.hidden = false;
    },
    readFile: ({ name, date }) => {
      readFrom.textContent = say("read-file", { name, date });
      readFrom.hidden = false;
    },
    failed: (text) => {
      lastError = text;
      update();
    },
    cleared: () => {
      lastError = "";
      update();
    },
    offer: (text, keep, drop) => {
      draft.querySelector("p").textContent = text;
      draft.querySelector(".draft-keep").textContent = keep.label;
      draft.querySelector(".draft-drop").textContent = drop.label;
      draft.hidden = false;
      const close = (action) => () => {
        draft.hidden = true;
        action();
        window.dispatchEvent(new Event("resize"));
      };
      draft.querySelector(".draft-keep").onclick = close(keep.action);
      draft.querySelector(".draft-drop").onclick = () => {
        if (window.confirm(say("draft-drop-confirm"))) close(drop.action)();
      };
      window.dispatchEvent(new Event("resize"));
    },
    download: root.querySelector(".download"),
    reload: root.querySelector(".reload"),
    file: root.querySelector(".reload-file"),
  };
};
