export default function title(t) {
  console.log("Title:", t);
  if (typeof document !== "undefined") {
    document.title = t;
  }
}

export let num = 0;

export const plusNum = () => {
  num++;
};
