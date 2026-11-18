import { Dayjs } from 'dayjs';
import MediaType from '../enums/MediaType';

export interface DownloadFilter {
  dateRange?: [start: Dayjs | null, end: Dayjs | null];
  mediaTypes?: MediaType[];
  source: 'medias' | 'tweets';
}
