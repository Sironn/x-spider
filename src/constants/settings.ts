import { Settings } from '../interfaces/Settings';

export const DEFAULT_SETTINGS: Settings = {
  proxy: {
    enable: true,
    url: 'http://127.0.0.1:7890',
    useSystem: false,
  },
  download: {
    saveDirBase: '',
    dirTemplate: '%USER_SCREEN_NAME%',
    fileNameTemplate:
      '%USER_SCREEN_NAME% [%POST_TIME%] %POST_ID%_%MEDIA_INDEX%%EXT%',
    sameFileSkip: true,
  },
  app: {
    autoCheckUpdate: true,
    acceptPrerelease: false,
    writeLogs: false,
  },
};

export const CURRENT_SETTINGS_VERSION = 2;
