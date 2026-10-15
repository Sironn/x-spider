/* eslint-disable react/prop-types */
import React from 'react';
import { clipboard } from '@tauri-apps/api';
import { Tooltip, App } from 'antd';
import { REPLACER_MAP } from '../../constants/file-name-template';

export const VariablePicker: React.FC = () => {
  const { message } = App.useApp();

  return (
    <section
      id="file-name-template-input-variables"
      aria-label="可用变量"
      className="mb-4 rounded-md border border-gray-200 bg-white p-4"
    >
      <details>
        <summary className="cursor-pointer select-none text-sm font-semibold">
          可用变量
        </summary>

        <div className="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4">
          {Object.entries(REPLACER_MAP).map(([k, v]) => (
            <Tooltip
              key={k}
              title={
                v.params ? (
                  <div>
                    <p>参数列表：</p>
                    <ul>
                      {v.params.map((param) => (
                        <li key={param.name}>
                          <strong>{param.name}</strong>
                          <span>：</span>
                          <span>{param.desc}</span>
                          <span>（默认：{param.default}）</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : undefined
              }
            >
              <button
                type="button"
                title="点击复制"
                className="flex min-h-[64px] w-full flex-col items-start justify-center rounded-lg border border-gray-200 bg-white px-3 py-2 text-left transition-colors hover:border-[#1677ff] hover:bg-[#f0f7ff] active:bg-[#e6f0ff]"
                onClick={() => {
                  clipboard.writeText(`%${k}%`).then(() => {
                    message.success('已复制到剪贴板');
                  });
                }}
              >
                <span className="break-all font-mono text-[13px] font-semibold text-[#1677ff] leading-tight">
                  %{k}%
                </span>
                <span className="mt-1 text-xs text-gray-500 leading-tight">
                  {v.desc}
                </span>
              </button>
            </Tooltip>
          ))}
        </div>

        <p className="mt-3 text-xs text-gray-500">
          参数格式：%VARIABLE,a=1,b=2%，如 %CONTENT,t=32% 代表推文内容截断 32
          字符。
        </p>
      </details>
    </section>
  );
};