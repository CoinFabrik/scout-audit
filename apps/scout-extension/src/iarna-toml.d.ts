declare module "@iarna/toml" {
  export interface TomlTable {
    [key: string]: unknown;
  }

  export function parse(content: string): TomlTable;
  export function stringify(obj: TomlTable): string;
}
