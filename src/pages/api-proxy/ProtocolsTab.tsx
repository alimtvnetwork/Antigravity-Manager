import { useMemo } from 'react';
import type { TFunction } from 'i18next';
import { CheckCircle, Copy, Terminal } from 'lucide-react';
import type { AppConfig } from '../../types/config';
import type { ProtocolKind, ProxyModelInfo, ProxyStatus } from './types';
import { DEFAULT_PROXY_PORT } from './constants';
import { ProtocolCards } from './ProtocolCards';

interface ProtocolsTabProps {
    config: AppConfig;
    t: TFunction;
    status: ProxyStatus;
    selectedProtocol: ProtocolKind;
    onSelectProtocol: (protocol: ProtocolKind) => void;
    selectedModelId: string;
    onSelectModelId: (modelId: string) => void;
    models: ProxyModelInfo[];
    copied: string | null;
    copyToClipboardHandler: (text: string, label: string) => void;
}

export function ProtocolsTab({
    config,
    t,
    status,
    selectedProtocol,
    onSelectProtocol,
    selectedModelId,
    onSelectModelId,
    models,
    copied,
    copyToClipboardHandler,
}: ProtocolsTabProps) {
    const getPythonExample = (modelId: string) => {
        const port = status.running ? status.port : (config.proxy.port || DEFAULT_PROXY_PORT);
        // 推荐使用 127.0.0.1 以避免部分环境 IPv6 解析延迟问题
        const baseUrl = `http://127.0.0.1:${port}/v1`;
        const apiKey = config.proxy.api_key || 'YOUR_API_KEY';

        // 1. Anthropic Protocol
        if (selectedProtocol === 'anthropic') {
            return `from anthropic import Anthropic

client = Anthropic(
    # Recommended: use 127.0.0.1
    base_url="${`http://127.0.0.1:${port}`}",
    api_key="${apiKey}"
)

# Note: Antigravity lets you call any model via the Anthropic SDK
response = client.messages.create(
    model="${modelId}",
    max_tokens=1024,
    messages=[{"role": "user", "content": "Hello"}]
)

print(response.content[0].text)`;
        }

        // 2. Gemini Protocol (Native)
        if (selectedProtocol === 'gemini') {
            const rawBaseUrl = `http://127.0.0.1:${port}`;
            return `# Requires: pip install google-generativeai
import google.generativeai as genai

# Use the Antigravity proxy address (recommended: 127.0.0.1)
genai.configure(
    api_key="${apiKey}",
    transport='rest',
    client_options={'api_endpoint': '${rawBaseUrl}'}
)

model = genai.GenerativeModel('${modelId}')
response = model.generate_content("Hello")
print(response.text)`;
        }

        // 3. OpenAI Protocol — image generation models (any *-image model)
        if (modelId.toLowerCase().includes('-image')) {
            return `from openai import OpenAI

client = OpenAI(
    base_url="${baseUrl}",
    api_key="${apiKey}"
)

# IMPORTANT — model availability:
#   "${modelId}" must be an image model your selected account actually has.
#   Check the model list on the left: not every account exposes every image model
#   (e.g. some accounts only have gemini-3.1-flash-image, not gemini-3-pro-image).
#   Requesting a model the account lacks fails with:
#     404 "Requested entity was not found"
#   To keep using a different name, add an explicit mapping in the Model Routing Center.

response = client.chat.completions.create(
    model="${modelId}",

    # Aspect ratio — Option 1: the size parameter (recommended)
    #   "1024x1024" = 1:1   |   "1280x720" = 16:9
    #   "720x1280"  = 9:16  |   "1216x896" = 4:3
    extra_body={ "size": "1024x1024" },

    # Aspect ratio — Option 2: a model-name suffix instead of size
    #   model="${modelId}-16-9"   (also: -9-16, -4-3, -3-4)
    messages=[{
        "role": "user",
        "content": "Draw a futuristic city"
    }]
)

# The generated image is returned INSIDE the message content
# (as a base64 data URL / markdown image), not as a hosted URL.
# Extract the base64 payload from here to save the file.
print(response.choices[0].message.content)`;
        }

        return `from openai import OpenAI

client = OpenAI(
    base_url="${baseUrl}",
    api_key="${apiKey}"
)

response = client.chat.completions.create(
    model="${modelId}",
    messages=[{"role": "user", "content": "Hello"}]
)

print(response.choices[0].message.content)`;
    };

    // 在 filter 逻辑中，当选择 openai 协议时，允许显示所有模型
    const filteredModels = useMemo(() => models.filter(model => {
        if (selectedProtocol === 'openai') {
            return true;
        }
        // Anthropic 协议下隐藏不支持的图片模型
        if (selectedProtocol === 'anthropic') {
            return !model.id.includes('image');
        }
        return true;
    }), [models, selectedProtocol]);

    return (
        <div className="p-4 space-y-6">
            <ProtocolCards
                config={config}
                t={t}
                status={status}
                selectedProtocol={selectedProtocol}
                onSelectProtocol={onSelectProtocol}
                copied={copied}
                copyToClipboardHandler={copyToClipboardHandler}
            />

            <div className="bg-white dark:bg-base-100 rounded-xl shadow-sm border border-gray-100 dark:border-base-200 overflow-hidden">
                <div className="px-4 py-2.5 border-b border-gray-100 dark:border-base-200">
                    <h2 className="text-base font-bold text-gray-900 dark:text-base-content flex items-center gap-2">
                        <Terminal size={18} />
                        {t('proxy.supported_models.title')}
                    </h2>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-0 lg:divide-x dark:divide-gray-700">
                    {/* 左侧：模型列表 */}
                    <div className="col-span-2 p-0">
                        <div className="overflow-x-auto">
                            <table className="table w-full">
                                <thead className="bg-gray-50/50 dark:bg-gray-800/50 text-gray-500 dark:text-gray-400">
                                    <tr>
                                        <th className="w-10 pl-3"></th>
                                        <th className="text-[11px] font-medium">{t('proxy.supported_models.model_name')}</th>
                                        <th className="text-[11px] font-medium">{t('proxy.supported_models.model_id')}</th>
                                        <th className="text-[11px] hidden sm:table-cell font-medium">{t('proxy.supported_models.description')}</th>
                                        <th className="text-[11px] w-20 text-center font-medium">{t('proxy.supported_models.action')}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {filteredModels.map((m) => (
                                        <tr
                                            key={m.id}
                                            className={`hover:bg-blue-50/50 dark:hover:bg-blue-900/10 cursor-pointer transition-colors ${selectedModelId === m.id ? 'bg-blue-50/80 dark:bg-blue-900/20' : ''}`}
                                            onClick={() => onSelectModelId(m.id)}
                                        >
                                            <td className="pl-4 text-blue-500">{m.icon}</td>
                                            <td className="font-bold text-xs">{m.name}</td>
                                            <td className="font-mono text-[10px] text-gray-500">{m.id}</td>
                                            <td className="text-[10px] text-gray-400 hidden sm:table-cell">{m.desc}</td>
                                            <td className="text-center">
                                                <button
                                                    className="btn btn-ghost btn-xs text-blue-500"
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        copyToClipboardHandler(m.id, `model-${m.id}`);
                                                    }}
                                                >
                                                    {copied === `model-${m.id}` ? <CheckCircle size={14} /> : <div className="flex items-center gap-1 text-[10px] font-bold tracking-tight"><Copy size={12} /> {t('common.copy')}</div>}
                                                </button>
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    </div>

                    {/* 右侧：代码预览 */}
                    <div className="col-span-1 bg-gray-900 text-blue-100 flex flex-col h-[400px] lg:h-auto">
                        <div className="p-3 border-b border-gray-800 flex items-center justify-between">
                            <span className="text-xs font-bold text-gray-400 uppercase tracking-wider">{t('proxy.multi_protocol.quick_integration')}</span>
                            <div className="flex gap-2">
                                <span className="text-[10px] px-2 py-0.5 rounded bg-blue-500/20 text-blue-400 border border-blue-500/30">
                                    {selectedProtocol === 'anthropic' ? 'Python (Anthropic SDK)' : (selectedProtocol === 'gemini' ? 'Python (Google GenAI)' : 'Python (OpenAI SDK)')}
                                </span>
                            </div>
                        </div>
                        <div className="flex-1 relative overflow-hidden group">
                            <div className="absolute inset-0 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-transparent">
                                <pre className="p-4 text-[10px] font-mono leading-relaxed">
                                    {getPythonExample(selectedModelId)}
                                </pre>
                            </div>
                            <button
                                onClick={() => copyToClipboardHandler(getPythonExample(selectedModelId), 'example-code')}
                                className="absolute top-4 right-4 p-2 bg-white/10 hover:bg-white/20 rounded-lg transition-colors text-white opacity-0 group-hover:opacity-100"
                            >
                                {copied === 'example-code' ? <CheckCircle size={16} /> : <Copy size={16} />}
                            </button>
                        </div>
                        <div className="p-3 bg-gray-800/50 border-t border-gray-800 text-[10px] text-gray-400">
                            {t('proxy.multi_protocol.click_tip')}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
