/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module "dplayer" {
  const DPlayer: {
    new (options?: any): any;
    prototype: any;
  };
  export default DPlayer;
}
