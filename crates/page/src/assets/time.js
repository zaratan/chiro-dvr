"use strict";
const VerdictsTime = (() => {
  const MINUTE = 60;
  const HOUR = 3600;
  const pad = (value) => String(value).padStart(2, "0");
  const whole = (seconds) => Math.max(Math.floor(seconds), 0);

  const clock = (seconds) => {
    const all = whole(seconds);
    return `${Math.floor(all / HOUR)}:${pad(Math.floor((all % HOUR) / MINUTE))}:${pad(all % MINUTE)}`;
  };

  const aloud = (seconds) => {
    const all = whole(seconds);
    const [hours, minutes, rest] = [Math.floor(all / HOUR), Math.floor((all % HOUR) / MINUTE), all % MINUTE];
    if (hours > 0) return `${hours} h ${minutes} min ${rest} s`;
    if (minutes > 0) return `${minutes} min ${rest} s`;
    return `${rest} s`;
  };

  const paired = (shown, said) => {
    const visible = document.createElement("span");
    visible.setAttribute("aria-hidden", "true");
    visible.textContent = shown;
    const spoken = document.createElement("span");
    spoken.className = "sr";
    spoken.textContent = said;
    return [visible, spoken];
  };

  return { clock, aloud, paired };
})();
