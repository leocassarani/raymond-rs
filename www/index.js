import { Image, Scene } from "raymond";
import { memory } from "raymond/raymond_bg";

const scene = new Scene();

const canvas = document.getElementById("canvas");
const image = new Image(canvas.width, canvas.height);

const length = canvas.width * canvas.height;
const pixels = new Uint8ClampedArray(memory.buffer, image.pixels(), length << 2);
const imageData = new ImageData(pixels, canvas.width, canvas.height);

const ctx = canvas.getContext('2d');

const render = () => {
  scene.render(image);
  ctx.putImageData(imageData, 0, 0);
};

document.addEventListener('keydown', e => {
  if (e.ctrlKey || e.altKey || e.metaKey) {
    return;
  }

  switch (e.key) {
    case 'a':
    case 'h':
      scene.moveLeft();
      break;
    case 'd':
    case 'l':
      scene.moveRight();
      break;
    case 'j':
      scene.moveDown();
      break;
    case 'k':
      scene.moveUp();
      break;
    case 'w':
      scene.moveForward();
      break;
    case 's':
      scene.moveBack();
      break;
    default:
      return;
  }

  e.preventDefault();
  render();
});

render();
