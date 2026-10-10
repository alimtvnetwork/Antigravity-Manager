import React from 'react';
import type { Dispatch, SetStateAction } from 'react';
import { useTranslation } from 'react-i18next';
import { RefreshCw } from 'lucide-react';
import type { UserToken } from './types';
import {
    EXPIRES_TYPE_DAY,
    EXPIRES_TYPE_WEEK,
    EXPIRES_TYPE_MONTH,
    EXPIRES_TYPE_CUSTOM,
    EXPIRES_TYPE_NEVER,
    DATETIME_LOCAL_TRUNCATE_LENGTH,
} from './types';

export interface TokenDialogsProps {
    showCreateModal: boolean;
    setShowCreateModal: Dispatch<SetStateAction<boolean>>;
    newUsername: string;
    setNewUsername: Dispatch<SetStateAction<string>>;
    newDesc: string;
    setNewDesc: Dispatch<SetStateAction<string>>;
    newExpiresType: string;
    setNewExpiresType: Dispatch<SetStateAction<string>>;
    newMaxIps: number;
    setNewMaxIps: Dispatch<SetStateAction<number>>;
    newCurfewStart: string;
    setNewCurfewStart: Dispatch<SetStateAction<string>>;
    newCurfewEnd: string;
    setNewCurfewEnd: Dispatch<SetStateAction<string>>;
    newCustomExpires: string;
    setNewCustomExpires: Dispatch<SetStateAction<string>>;
    creating: boolean;
    handleCreate: () => Promise<void>;
    showEditModal: boolean;
    setShowEditModal: Dispatch<SetStateAction<boolean>>;
    editingToken: UserToken | null;
    editUsername: string;
    setEditUsername: Dispatch<SetStateAction<string>>;
    editDesc: string;
    setEditDesc: Dispatch<SetStateAction<string>>;
    editMaxIps: number;
    setEditMaxIps: Dispatch<SetStateAction<number>>;
    editCurfewStart: string;
    setEditCurfewStart: Dispatch<SetStateAction<string>>;
    editCurfewEnd: string;
    setEditCurfewEnd: Dispatch<SetStateAction<string>>;
    updating: boolean;
    handleUpdate: () => Promise<void>;
}

export const TokenDialogs: React.FC<TokenDialogsProps> = ({
    showCreateModal,
    setShowCreateModal,
    newUsername,
    setNewUsername,
    newDesc,
    setNewDesc,
    newExpiresType,
    setNewExpiresType,
    newMaxIps,
    setNewMaxIps,
    newCurfewStart,
    setNewCurfewStart,
    newCurfewEnd,
    setNewCurfewEnd,
    newCustomExpires,
    setNewCustomExpires,
    creating,
    handleCreate,
    showEditModal,
    setShowEditModal,
    editingToken,
    editUsername,
    setEditUsername,
    editDesc,
    setEditDesc,
    editMaxIps,
    setEditMaxIps,
    editCurfewStart,
    setEditCurfewStart,
    editCurfewEnd,
    setEditCurfewEnd,
    updating,
    handleUpdate,
}) => {
    const { t } = useTranslation();

    return (
        <>
            {/* Create Modal */}
            {showCreateModal && (
                <div className="modal modal-open">
                    <div className="modal-box">
                        <h3 className="font-bold text-lg mb-4">{t('user_token.create_title', { defaultValue: 'Create New Token' })}</h3>

                        <div className="form-control w-full mb-3">
                            <label className="label">
                                <span className="label-text">{t('user_token.username', { defaultValue: 'Username' })} *</span>
                            </label>
                            <input
                                type="text"
                                className="input input-bordered w-full"
                                value={newUsername}
                                onChange={e => setNewUsername(e.target.value)}
                                placeholder={t('user_token.placeholder_username', { defaultValue: 'e.g. user1' })}
                            />
                        </div>

                        <div className="form-control w-full mb-3">
                            <label className="label">
                                <span className="label-text">{t('user_token.description', { defaultValue: 'Description' })}</span>
                            </label>
                            <input
                                type="text"
                                className="input input-bordered w-full"
                                value={newDesc}
                                onChange={e => setNewDesc(e.target.value)}
                                placeholder={t('user_token.placeholder_desc', { defaultValue: 'Optional notes' })}
                            />
                        </div>

                        <div className="grid grid-cols-2 gap-4 mb-3">
                            <div className="form-control w-full">
                                <label className="label">
                                    <span className="label-text">{t('user_token.expires', { defaultValue: 'Expires In' })}</span>
                                </label>
                                <select
                                    className="select select-bordered w-full"
                                    value={newExpiresType}
                                    onChange={e => setNewExpiresType(e.target.value)}
                                >
                                    <option value={EXPIRES_TYPE_DAY}>{t('user_token.expires_day', { defaultValue: '1 Day' })}</option>
                                    <option value={EXPIRES_TYPE_WEEK}>{t('user_token.expires_week', { defaultValue: '1 Week' })}</option>
                                    <option value={EXPIRES_TYPE_MONTH}>{t('user_token.expires_month', { defaultValue: '1 Month' })}</option>
                                    <option value={EXPIRES_TYPE_CUSTOM}>{t('user_token.expires_custom', { defaultValue: 'Custom' })}</option>
                                    <option value={EXPIRES_TYPE_NEVER}>{t('user_token.expires_never', { defaultValue: 'Never' })}</option>
                                </select>
                            </div>

                            <div className="form-control w-full">
                                <label className="label">
                                    <span className="label-text">{t('user_token.ip_limit', { defaultValue: 'Max IPs' })}</span>
                                </label>
                                <input
                                    type="number"
                                    className="input input-bordered w-full"
                                    value={newMaxIps}
                                    onChange={e => setNewMaxIps(parseInt(e.target.value) || 0)}
                                    min="0"
                                    placeholder={t('user_token.placeholder_max_ips', { defaultValue: '0 = Unlimited' })}
                                />
                                <label className="label">
                                    <span className="label-text-alt text-gray-500">{t('user_token.hint_max_ips', { defaultValue: '0 = Unlimited' })}</span>
                                </label>
                            </div>
                        </div>

                        {/* Custom Expiration Time Picker */}
                        {newExpiresType === EXPIRES_TYPE_CUSTOM && (
                            <div className="form-control w-full mb-3">
                                <label className="label">
                                    <span className="label-text">{t('user_token.custom_expires_at', { defaultValue: 'Expiration Date & Time' })} *</span>
                                </label>
                                <input
                                    type="datetime-local"
                                    className="input input-bordered w-full"
                                    value={newCustomExpires}
                                    onChange={e => setNewCustomExpires(e.target.value)}
                                    min={new Date().toISOString().slice(0, DATETIME_LOCAL_TRUNCATE_LENGTH)}
                                />
                                <label className="label">
                                    <span className="label-text-alt text-gray-500">{t('user_token.hint_custom_expires', { defaultValue: 'Select the exact date and hour when this token expires' })}</span>
                                </label>
                            </div>
                        )}

                        <div className="form-control w-full mb-3">
                            <label className="label">
                                <span className="label-text">{t('user_token.curfew', { defaultValue: 'Curfew (Service Unavailable Time)' })}</span>
                            </label>
                            <div className="flex gap-2 items-center">
                                <input
                                    type="time"
                                    className="input input-bordered w-full"
                                    value={newCurfewStart}
                                    onChange={e => setNewCurfewStart(e.target.value)}
                                />
                                <span className="text-gray-400">to</span>
                                <input
                                    type="time"
                                    className="input input-bordered w-full"
                                    value={newCurfewEnd}
                                    onChange={e => setNewCurfewEnd(e.target.value)}
                                />
                            </div>
                            <label className="label">
                                <span className="label-text-alt text-gray-500">{t('user_token.hint_curfew', { defaultValue: 'Leave empty to disable. Based on Beijing time (UTC+8).' })}</span>
                            </label>
                        </div>

                        <div className="modal-action">
                            <button className="px-4 py-2 hover:bg-gray-100 dark:hover:bg-base-200 rounded-lg text-sm transition-colors" onClick={() => setShowCreateModal(false)}>
                                {t('common.cancel', { defaultValue: 'Cancel' })}
                            </button>
                            <button
                                className={`px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium rounded-lg transition-all shadow-sm shadow-blue-500/20 flex items-center gap-2 ${creating ? 'opacity-50 cursor-not-allowed' : ''}`}
                                onClick={handleCreate}
                                disabled={creating}
                            >
                                {creating && <RefreshCw size={14} className="animate-spin" />}
                                {t('common.create', { defaultValue: 'Create' })}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Edit Modal */}
            {showEditModal && editingToken && (
                <div className="modal modal-open">
                    <div className="modal-box">
                        <h3 className="font-bold text-lg mb-4">{t('user_token.edit_title', { defaultValue: 'Edit Token' })}</h3>

                        <div className="form-control w-full mb-3">
                            <label className="label">
                                <span className="label-text">{t('user_token.username', { defaultValue: 'Username' })} *</span>
                            </label>
                            <input
                                type="text"
                                className="input input-bordered w-full"
                                value={editUsername}
                                onChange={e => setEditUsername(e.target.value)}
                                placeholder={t('user_token.placeholder_username', { defaultValue: 'e.g. user1' })}
                            />
                        </div>

                        <div className="form-control w-full mb-3">
                            <label className="label">
                                <span className="label-text">{t('user_token.description', { defaultValue: 'Description' })}</span>
                            </label>
                            <input
                                type="text"
                                className="input input-bordered w-full"
                                value={editDesc}
                                onChange={e => setEditDesc(e.target.value)}
                                placeholder={t('user_token.placeholder_desc', { defaultValue: 'Optional notes' })}
                            />
                        </div>

                        <div className="form-control w-full mb-3">
                            <label className="label">
                                <span className="label-text">{t('user_token.ip_limit', { defaultValue: 'Max IPs' })}</span>
                            </label>
                            <input
                                type="number"
                                className="input input-bordered w-full"
                                value={editMaxIps}
                                onChange={e => setEditMaxIps(parseInt(e.target.value) || 0)}
                                min="0"
                                placeholder={t('user_token.placeholder_max_ips', { defaultValue: '0 = Unlimited' })}
                            />
                            <label className="label">
                                <span className="label-text-alt text-gray-500">{t('user_token.hint_max_ips', { defaultValue: '0 = Unlimited' })}</span>
                            </label>
                        </div>

                        <div className="form-control w-full mb-3">
                            <label className="label">
                                <span className="label-text">{t('user_token.curfew', { defaultValue: 'Curfew (Service Unavailable Time)' })}</span>
                            </label>
                            <div className="flex gap-2 items-center">
                                <input
                                    type="time"
                                    className="input input-bordered w-full"
                                    value={editCurfewStart}
                                    onChange={e => setEditCurfewStart(e.target.value)}
                                />
                                <span className="text-gray-400">to</span>
                                <input
                                    type="time"
                                    className="input input-bordered w-full"
                                    value={editCurfewEnd}
                                    onChange={e => setEditCurfewEnd(e.target.value)}
                                />
                            </div>
                            <label className="label">
                                <span className="label-text-alt text-gray-500">{t('user_token.hint_curfew', { defaultValue: 'Leave empty to disable. Based on Beijing time (UTC+8).' })}</span>
                            </label>
                        </div>

                        <div className="modal-action">
                            <button className="px-4 py-2 hover:bg-gray-100 dark:hover:bg-base-200 rounded-lg text-sm transition-colors" onClick={() => setShowEditModal(false)}>
                                {t('common.cancel', { defaultValue: 'Cancel' })}
                            </button>
                            <button
                                className={`px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium rounded-lg transition-all shadow-sm shadow-blue-500/20 flex items-center gap-2 ${updating ? 'opacity-50 cursor-not-allowed' : ''}`}
                                onClick={handleUpdate}
                                disabled={updating}
                            >
                                {updating && <RefreshCw size={14} className="animate-spin" />}
                                {t('common.update', { defaultValue: 'Update' })}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
};
