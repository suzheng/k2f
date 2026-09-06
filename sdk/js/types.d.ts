/** Public TypeScript surface for the `@openk2f/k2f` and `@openk2f/k2f/viewer` package entries. */

export function wrapWasm(wasm: WasmModule): K2fApi;

export function initWasm(wasmSource?: InitWasmSource): Promise<WasmModule>;
export function initViewerWasm(wasmSource?: InitWasmSource): Promise<ViewerWasmModule>;
export function createK2f(): Promise<K2fApi>;
export function createViewer(): Promise<ViewerOnlyApi>;
export function exportPdf(packageBytes: Uint8Array): Promise<Uint8Array>;
export function exportPptx(packageBytes: Uint8Array): Promise<Uint8Array>;
export function exportDocx(packageBytes: Uint8Array): Promise<Uint8Array>;
export function markdownToK2f(
  md: string,
  opts?: { title?: string; template?: string },
): Promise<Uint8Array>;
export function k2fToMarkdown(bytes: Uint8Array): Promise<string>;

export type InitWasmSource = string | URL | Request | BufferSource | WebAssembly.Module;

export interface ViewerWasmModule {
  default: (module_or_path?: unknown) => Promise<unknown>;
  K2fViewer: typeof Viewer;
}

export interface ViewerOnlyApi {
  Viewer: typeof Viewer;
}

export interface WasmModule {
  default: (module_or_path?: unknown) => Promise<unknown>;
  K2fEditor: {
    new (bytes: Uint8Array): unknown;
    openTemplate(template: string): unknown;
  };
  K2fViewer: typeof Viewer;
  generate_signing_key: () => string;
  sign_k2f: (
    bytes: Uint8Array,
    secretHex: string,
    signedBy?: string | null,
    signedAt?: bigint | null,
  ) => Uint8Array;
  official_templates: () => string;
  resolve_template: (template: string) => string;
  copy_template: (template: string, dest: string) => void;
  markdown_to_k2f: (md: string, title: string, template: string) => Uint8Array;
  k2f_to_markdown: (bytes: Uint8Array) => string;
}

export interface GeneratedKey {
  secret_hex: string;
  public_hex: string;
  fingerprint: string;
}

export interface K2fApi {
  systemPrompt(): string;
  officialTemplates(): string[];
  resolveTemplate(template: string): string;
  copyTemplate(template: string, dest: string): void;
  Editor: typeof Editor;
  Viewer: typeof Viewer;
  generateSigningKey(): GeneratedKey;
  sign(
    bytes: Uint8Array,
    secretHex: string,
    signedBy?: string | null,
    signedAt?: bigint | null,
  ): Uint8Array;
  markdownToK2f(md: string, opts?: { title?: string; template?: string }): Uint8Array;
  k2fToMarkdown(bytes: Uint8Array): string;
}

export class Editor {
  static open(bytes: Uint8Array): Editor;
  static openTemplate(template: string): Editor;
  outline(): OutlineNode[];
  diff(): SemanticChange[];
  getNode(id: string): unknown;
  insertNode(parentId: string, index: number, node: string | object): void;
  deleteNode(id: string): void;
  setGeneratedBy(id: string): void;
  setRunningHeader(text: string): void;
  set_running_header(text: string): void;
  setRunningFooter(text: string): void;
  set_running_footer(text: string): void;
  selection(id: string): SelectionJson;
  clipboard(id: string): unknown;
  replaceText(id: string, text: string): void;
  replace_text(id: string, text: string): void;
  setRole(id: string, role: string, variant?: string | null): void;
  set_role(id: string, role: string, variant?: string | null): void;
  search(query: string): string[];
  suggestions(): Record<string, string>;
  suggest(id: string, text: string): void;
  acceptSuggestion(id: string): void;
  accept_suggestion(id: string): void;
  rejectSuggestion(id: string): void;
  reject_suggestion(id: string): void;
  save(): Uint8Array;
  saveWith(expectedContentHash?: string | null): Uint8Array;
  exportPptx(): Uint8Array;
  export_pptx(): Uint8Array;
  exportDocx(): Uint8Array;
  export_docx(): Uint8Array;
  free(): void;
}

export interface OutlineNode {
  id: string;
  role: string;
  preview?: string;
  children: string[];
}

export interface SemanticChange {
  id: string;
  op: "add" | "delete" | "update";
  text_before?: string;
  text_after?: string;
  role_before?: string;
  role_after?: string;
}

export class Viewer {
  constructor(bytes: Uint8Array);
  free(): void;
  static official_scale(): number;
  banner(): string;
  status_code(): string;
  hash_code(): string;
  fingerprint(): string | undefined;
  signed_by(): string | undefined;
  signed_at(): bigint | undefined;
  generated_by(): string | undefined;
  content_hash(): string | undefined;
  appearance_hash(): string | undefined;
  page_count(): number;
  title(): string;
  page_width_pt(page: number): number;
  page_height_pt(page: number): number;
  render_page(page: number, scale: number): Uint8Array;
  export_pdf(): Uint8Array;
  export_pptx(): Uint8Array;
  export_docx(): Uint8Array;
  search(query: string): string;
  hit_test(page: number, x_pt: number, y_pt: number): string | undefined;
  hit_selection(page: number, x_pt: number, y_pt: number): string | undefined;
  boxes_for(id: string): string;
  clipboard(id: string): string | undefined;
  selection(id: string): string | undefined;
  text_layer(page: number): string;
  selection_markdown(rangesJson: string): string;
  document_markdown(): string;
  export_pages_png_zip(scale: number): Uint8Array;
  export_pages_jpeg_zip(scale: number): Uint8Array;
}

export interface SelectionJson {
  id: string;
  ids?: string[];
  role?: string;
  variant?: string;
  text?: string | null;
  char_range?: [number, number] | null;
  node?: unknown;
}

export interface TextSpanJson {
  node_id: string;
  char_start: number;
  char_end: number;
  text: string;
  x_pt: number;
  y_pt: number;
  width_pt: number;
  height_pt: number;
}

export interface ViewerMountOptions {
  /** Integrity banner presentation. Default `auto` (tiered). `true`/`full` = legacy verbose strip. `false`/`off` = hidden. */
  banner?: boolean | "auto" | "full" | "off";
  /** Allow Edit toolbar + surgical popover (opens in view mode). Forces sdk WASM. */
  editable?: boolean;
  /** Explicit WASM flavor. `editable: true` always uses sdk. Default is viewer when available. */
  runtime?: "sdk" | "viewer";
  copyFormat?: "markdown" | "plain";
  exportFormat?: "k2f" | "pdf" | "pptx" | "docx" | "markdown" | "png" | "jpg";
  title?: string;
}

export interface ViewerHandle {
  viewer: Viewer;
  editor: Editor | null;
  /** True when the toolbar Edit toggle is active. */
  editing: boolean;
  destroy(): void;
  goPage(page: number): void;
  export(format?: "k2f" | "pdf" | "pptx" | "docx" | "markdown" | "png" | "jpg"): {
    bytes: Uint8Array;
    filename: string;
    mime: string;
  };
  open(bytes: Uint8Array): Promise<void>;
  selectId(id: string): void;
}

export interface ViewerOpenEvent {
  banner: string;
  statusCode: string;
  pageCount: number;
  handle: ViewerHandle;
}

export function mountK2fViewer(
  host: HTMLElement,
  bytes: Uint8Array,
  options?: ViewerMountOptions,
): Promise<ViewerHandle>;

export class K2fViewerElement extends HTMLElement {
  static readonly observedAttributes: string[];
  src?: string;
  editable?: boolean;
  /** Hide the integrity banner when present. Same as `banner="off"`. */
  "no-banner"?: boolean;
  readonly handle?: ViewerHandle;
  open(bytes: Uint8Array): Promise<void>;
}
