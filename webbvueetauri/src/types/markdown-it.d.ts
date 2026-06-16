declare module 'markdown-it' {
  interface MarkdownItOptions {
    html?: boolean;
    xhtmlOut?: boolean;
    breaks?: boolean;
    langPrefix?: string;
    linkify?: boolean;
    typographer?: boolean;
    quotes?: string | string[];
    highlight?: (str: string, lang: string, attrs: string) => string;
  }

  interface MarkdownIt {
    render(src: string, env?: any): string;
    renderInline(src: string, env?: any): string;
  }

  class MarkdownIt {
    constructor(options?: MarkdownItOptions);
    render(src: string, env?: any): string;
    renderInline(src: string, env?: any): string;
  }

  export = MarkdownIt;
}
