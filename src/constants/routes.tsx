import { Route } from '../interfaces/Route';
import {
  HomeFilled,
  SettingFilled,
  DownloadOutlined,
} from '@ant-design/icons';
import { Homepage } from '../pages/Homepage';
import { DownloadManagement } from '../pages/DownloadManagement';
import { Settings } from '../pages/Settings';

export const ROUTES: Route[] = [
  {
    id: 'home',
    name: '主页',
    icon: <HomeFilled />,
    element: <Homepage />,
  },
  {
    id: 'download-management',
    name: '下载管理',
    icon: <DownloadOutlined />,
    element: <DownloadManagement />,
  },
  {
    id: 'settings',
    name: '设置',
    icon: <SettingFilled />,
    element: <Settings />,
  },
];
