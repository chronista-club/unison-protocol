// Auto-generated TypeScript protocol definitions
// DO NOT EDIT MANUALLY

export type UUID = string;
export type Timestamp = string;
export type LanguageCode = string;

export interface TranslationSession {
  id: UUID;
  name: string;
  status: 'draft' | 'pending' | 'in_progress' | 'completed' | 'failed' | 'archived';
  created_at: Timestamp;
}

export interface Translation {
  id: UUID;
  source_text: string;
  target_text: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  created_at: Timestamp;
}
