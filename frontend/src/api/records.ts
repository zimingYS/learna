import { request } from "./http";


export interface RecordItem {
  id: string;
  title: string;
  content: string | null;
  duration_minutes: number | null;
  subject: string | null;
  created_at: string;
  updated_at: string;
}

export interface RecordListData {
  items: RecordItem[];
  total: number;
}

export interface CreateRecordInput {
  title: string;
  content?: string;
  duration_minutes?: number;
  subject?: string;
}

export interface UpdateRecordInput {
  title?: string;
  content?: string;
  duration_minutes?: number;
  subject?: string;
}

export function fetchRecords(page: number, pageSize: number): Promise<RecordListData> {
  return request<RecordListData>({
    url: "/records",
    method: "GET",
    params: {page, page_size: pageSize},
  });
}

export function fetchRecord(id: string): Promise<RecordItem> {
  return request<RecordItem>({
    url: `/records/${id}`,
    method: "GET",
  });
}

export function createRecord(input: CreateRecordInput): Promise<RecordItem> {
  return request<RecordItem>({
    url: "/records",
    method: "POST",
    data: input,
  });
}

export function updateRecord(id: string, input: UpdateRecordInput): Promise<RecordItem> {
  return request<RecordItem>({
    url: `/records/${id}`,
    method: "PUT",
    data: input,
  });
}

export async function deleteRecord(id: string): Promise<void> {
  await request<void>({
    url: `/records/${id}`,
    method: "DELETE",
  });
}
