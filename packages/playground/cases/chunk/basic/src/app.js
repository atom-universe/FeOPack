export default function title(t) {
  console.log("Title:", t);
  if (typeof document !== "undefined") {
    document.title = t;
  }
}

