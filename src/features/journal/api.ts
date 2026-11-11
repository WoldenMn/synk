import { invoke } from "@tauri-apps/api/core";

export interface JournalEntry {
  id: number;
  date: string;
  content: string;
  tags: string | null;
  created_at: string;
  updated_at: string;
}

export interface JournalInput {
  date: string;
  content: string;
  tags?: string | null;
}

export function getJournalEntries(from?: string, to?: string): Promise<JournalEntry[]> {
  return invoke<JournalEntry[]>("get_journal_entries", { from, to });
}

export function upsertJournalEntry(data: JournalInput): Promise<number> {
  return invoke<number>("upsert_journal_entry", { data });
}

export function deleteJournalEntry(id: number): Promise<void> {
  return invoke("delete_journal_entry", { id });
}
