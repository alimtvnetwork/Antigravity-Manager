import React from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, RefreshCw, Copy, Activity, User, Settings, Shield, Clock, Users } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { useUserTokens } from './user-token/useUserTokens';
import { TokenDialogs } from './user-token/TokenDialogs';
import {
    TOKEN_PREVIEW_LENGTH,
    TOKEN_PREVIEW_MASK,
    USERNAME_AVATAR_LENGTH,
    TOKENS_PER_THOUSAND,
    TOKEN_KILO_DECIMALS,
    MS_PER_SECOND,
    EXPIRES_TYPE_DAY,
    EXPIRES_TYPE_WEEK,
    EXPIRES_TYPE_MONTH,
} from './user-token/types';

const UserToken: React.FC = () => {
    const { t } = useTranslation();
    const state = useUserTokens();
    const {
        tokens,
        stats,
        loading,
        setShowCreateModal,
        loadData,
        handleDelete,
        handleEdit,
        handleRenew,
        handleCopyToken,
        formatTime,
        getExpiresLabel,
        getExpiresStatus,
    } = state;

    return (
        <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="h-full flex flex-col px-4 sm:px-6 pt-2 pb-4 gap-4 max-w-[1920px] mx-auto w-full"
        >
            {/* Header */}
            <div className="flex justify-between items-center">
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                    <div className="p-2 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
                        <User className="text-purple-500 w-5 h-5" />
                    </div>
                    {t('user_token.title', { defaultValue: 'User Tokens' })}
                </h1>

                <div className="flex items-center gap-2">
                    <button
                        onClick={() => loadData()}
                        className={`p-2 hover:bg-gray-100 dark:hover:bg-base-200 rounded-lg transition-colors ${loading ? 'text-blue-500' : 'text-gray-500'}`}
                        title={t('common.refresh') || 'Refresh'}
                    >
                        <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
                    </button>
                    <button
                        onClick={() => setShowCreateModal(true)}
                        className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium rounded-lg transition-all flex items-center gap-2 shadow-sm shadow-blue-500/20"
                    >
                        <Plus size={16} />
                        <span>{t('user_token.create', { defaultValue: 'Create Token' })}</span>
                    </button>
                </div>
            </div>

            {/* Stats Cards Row */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200"
                >
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-1.5 bg-blue-50 dark:bg-blue-900/20 rounded-md">
                            <Users className="w-4 h-4 text-blue-500" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-base-content mb-0.5">{stats?.total_users || 0}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">{t('user_token.total_users', { defaultValue: 'Total Users' })}</div>
                </motion.div>

                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200"
                >
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-1.5 bg-green-50 dark:bg-green-900/20 rounded-md">
                            <Activity className="w-4 h-4 text-green-500" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-base-content mb-0.5">{stats?.active_tokens || 0}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">{t('user_token.active_tokens', { defaultValue: 'Active Tokens' })}</div>
                </motion.div>

                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200"
                >
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-1.5 bg-purple-50 dark:bg-purple-900/20 rounded-md">
                            <Clock className="w-4 h-4 text-purple-500" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-base-content mb-0.5">{stats?.total_tokens || 0}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">{t('user_token.total_created', { defaultValue: 'Total Tokens' })}</div>
                </motion.div>

                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200"
                >
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-1.5 bg-orange-50 dark:bg-orange-900/20 rounded-md">
                            <Shield className="w-4 h-4 text-orange-500" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-base-content mb-0.5">{stats?.today_requests || 0}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">{t('user_token.today_requests', { defaultValue: 'Today Requests' })}</div>
                </motion.div>
            </div>

            {/* Token List */}
            <div className="flex-1 overflow-auto bg-white dark:bg-base-100 rounded-2xl shadow-sm border border-gray-100 dark:border-base-200">
                <table className="table table-pin-rows">
                    <thead>
                        <tr className="bg-gray-50/50 dark:bg-base-200/50">
                            <th className="bg-transparent text-gray-500 font-medium py-4">{t('user_token.username', { defaultValue: 'Username' })}</th>
                            <th className="bg-transparent text-gray-500 font-medium py-4">{t('user_token.token', { defaultValue: 'Token' })}</th>
                            <th className="bg-transparent text-gray-500 font-medium py-4">{t('user_token.expires', { defaultValue: 'Expires' })}</th>
                            <th className="bg-transparent text-gray-500 font-medium py-4">{t('user_token.usage', { defaultValue: 'Usage' })}</th>
                            <th className="bg-transparent text-gray-500 font-medium py-4">{t('user_token.ip_limit', { defaultValue: 'IP Limit' })}</th>
                            <th className="bg-transparent text-gray-500 font-medium py-4">{t('user_token.created', { defaultValue: 'Created' })}</th>
                            <th className="bg-transparent text-gray-500 font-medium py-4 text-right">{t('common.actions', { defaultValue: 'Actions' })}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-50 dark:divide-base-200">
                        <AnimatePresence mode="popLayout">
                            {tokens.map((token, index) => (
                                <motion.tr
                                    key={token.id}
                                    initial={{ opacity: 0, x: -10 }}
                                    animate={{ opacity: 1, x: 0 }}
                                    exit={{ opacity: 0, scale: 0.95 }}
                                    transition={{ delay: index * 0.03 }}
                                    className="hover:bg-gray-50/80 dark:hover:bg-base-200/50 transition-colors group"
                                >
                                    <td className="py-4">
                                        <div className="flex items-center gap-3">
                                            <div className="w-8 h-8 rounded-full bg-purple-50 dark:bg-purple-900/20 flex items-center justify-center text-purple-600 font-bold text-xs">
                                                {token.username.substring(0, USERNAME_AVATAR_LENGTH).toUpperCase()}
                                            </div>
                                            <div>
                                                <div className="font-semibold text-gray-900 dark:text-white uppercase tracking-wider text-xs">{token.username}</div>
                                                <div className="text-[10px] text-gray-500">{token.description || '-'}</div>
                                            </div>
                                        </div>
                                    </td>
                                    <td>
                                        <div className="flex items-center gap-2 group/token">
                                            <code className="bg-gray-50 dark:bg-base-200 px-2 py-1 rounded border border-gray-100 dark:border-base-300 text-[11px] font-mono text-gray-600 dark:text-gray-400">
                                                {token.token.substring(0, TOKEN_PREVIEW_LENGTH)}{TOKEN_PREVIEW_MASK}
                                            </code>
                                            <button
                                                onClick={() => handleCopyToken(token.token)}
                                                className="p-1.5 hover:bg-gray-200 dark:hover:bg-base-300 rounded-md transition-all text-gray-400 hover:text-gray-600 dark:hover:text-white"
                                            >
                                                <Copy size={13} />
                                            </button>
                                        </div>
                                    </td>
                                    <td>
                                        <div className={`text-xs font-medium mb-1 ${getExpiresStatus(token.expires_at)}`}>
                                            {token.expires_at ? formatTime(token.expires_at) : t('user_token.never', { defaultValue: 'Never' })}
                                        </div>
                                        <div className="flex items-center gap-2">
                                            <span className="text-[10px] px-1.5 py-0.5 bg-gray-100 dark:bg-base-200 text-gray-500 rounded lowercase">
                                                {getExpiresLabel(token.expires_type)}
                                            </span>
                                            {token.expires_at && token.expires_at < Date.now() / MS_PER_SECOND && (
                                                <button
                                                    onClick={() => handleRenew(token.id, token.expires_type)}
                                                    className="text-[10px] text-blue-500 hover:underline font-medium"
                                                >
                                                    {t('user_token.renew_button', { defaultValue: 'Renew' })}
                                                </button>
                                            )}
                                        </div>
                                    </td>
                                    <td>
                                        <div className="text-xs font-semibold text-gray-700 dark:text-gray-300">{token.total_requests} <span className="text-[10px] font-normal text-gray-400">reqs</span></div>
                                        <div className="text-[10px] text-gray-400 mt-0.5">
                                            {(token.total_tokens_used / TOKENS_PER_THOUSAND).toFixed(TOKEN_KILO_DECIMALS)}k tokens
                                        </div>
                                    </td>
                                    <td>
                                        {token.max_ips === 0
                                            ? <span className="px-2 py-0.5 bg-gray-100 dark:bg-base-200 text-gray-500 text-[10px] rounded-full">{t('user_token.unlimited', { defaultValue: 'Unlimited' })}</span>
                                            : <span className="px-2 py-0.5 bg-orange-50 dark:bg-orange-900/20 text-orange-600 dark:text-orange-400 text-[10px] font-medium rounded-full border border-orange-100 dark:border-orange-900/30">{token.max_ips} IPs</span>
                                        }
                                        {token.curfew_start && token.curfew_end && (
                                            <div className="text-[10px] text-gray-400 mt-1.5 flex items-center gap-1 bg-gray-50 dark:bg-base-200 w-fit px-1.5 py-0.5 rounded">
                                                <Clock size={10} className="text-orange-500" />
                                                <span>{token.curfew_start} - {token.curfew_end}</span>
                                            </div>
                                        )}
                                    </td>
                                    <td className="text-[10px] text-gray-400 italic">
                                        {formatTime(token.created_at)}
                                    </td>
                                    <td className="text-right">
                                        <div className="flex justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                            <button
                                                onClick={() => handleEdit(token)}
                                                className="p-1.5 hover:bg-gray-100 dark:hover:bg-base-200 rounded-lg text-gray-500 hover:text-blue-500 transition-colors"
                                                title={t('common.edit', { defaultValue: 'Edit' })}
                                            >
                                                <Settings size={14} />
                                            </button>
                                            <div className="dropdown dropdown-end">
                                                <label tabIndex={0} className="p-1.5 hover:bg-gray-100 dark:hover:bg-base-200 rounded-lg text-gray-500 hover:text-green-500 transition-colors inline-block cursor-pointer">
                                                    <RefreshCw size={14} />
                                                </label>
                                                <ul tabIndex={0} className="dropdown-content z-[10] menu p-2 shadow-xl bg-white dark:bg-base-100 rounded-xl w-32 border border-gray-100 dark:border-base-200 mt-1">
                                                    <div className="px-3 py-1.5 text-[10px] font-bold text-gray-400 uppercase tracking-widest">{t('user_token.renew')}</div>
                                                    <li><a className="text-xs py-2" onClick={() => handleRenew(token.id, EXPIRES_TYPE_DAY)}>{t('user_token.expires_day', { defaultValue: '1 Day' })}</a></li>
                                                    <li><a className="text-xs py-2" onClick={() => handleRenew(token.id, EXPIRES_TYPE_WEEK)}>{t('user_token.expires_week', { defaultValue: '1 Week' })}</a></li>
                                                    <li><a className="text-xs py-2" onClick={() => handleRenew(token.id, EXPIRES_TYPE_MONTH)}>{t('user_token.expires_month', { defaultValue: '1 Month' })}</a></li>
                                                </ul>
                                            </div>
                                            <button
                                                onClick={() => handleDelete(token.id)}
                                                className="p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg text-gray-400 hover:text-red-500 transition-colors"
                                            >
                                                <Trash2 size={14} />
                                            </button>
                                        </div>
                                    </td>
                                </motion.tr>
                            ))}
                        </AnimatePresence>
                        {tokens.length === 0 && !loading && (
                            <tr>
                                <td colSpan={7} className="py-20">
                                    <div className="flex flex-col items-center justify-center text-gray-400 gap-3">
                                        <div className="p-4 bg-gray-50 dark:bg-base-200 rounded-full">
                                            <Users size={40} className="opacity-20" />
                                        </div>
                                        <p className="text-sm">{t('user_token.no_data', { defaultValue: 'No tokens found' })}</p>
                                        <button
                                            onClick={() => setShowCreateModal(true)}
                                            className="text-xs text-blue-500 hover:underline"
                                        >
                                            {t('user_token.create', { defaultValue: 'Create your first token' })}
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>

            <TokenDialogs
                showCreateModal={state.showCreateModal}
                setShowCreateModal={state.setShowCreateModal}
                newUsername={state.newUsername}
                setNewUsername={state.setNewUsername}
                newDesc={state.newDesc}
                setNewDesc={state.setNewDesc}
                newExpiresType={state.newExpiresType}
                setNewExpiresType={state.setNewExpiresType}
                newMaxIps={state.newMaxIps}
                setNewMaxIps={state.setNewMaxIps}
                newCurfewStart={state.newCurfewStart}
                setNewCurfewStart={state.setNewCurfewStart}
                newCurfewEnd={state.newCurfewEnd}
                setNewCurfewEnd={state.setNewCurfewEnd}
                newCustomExpires={state.newCustomExpires}
                setNewCustomExpires={state.setNewCustomExpires}
                creating={state.creating}
                handleCreate={state.handleCreate}
                showEditModal={state.showEditModal}
                setShowEditModal={state.setShowEditModal}
                editingToken={state.editingToken}
                editUsername={state.editUsername}
                setEditUsername={state.setEditUsername}
                editDesc={state.editDesc}
                setEditDesc={state.setEditDesc}
                editMaxIps={state.editMaxIps}
                setEditMaxIps={state.setEditMaxIps}
                editCurfewStart={state.editCurfewStart}
                setEditCurfewStart={state.setEditCurfewStart}
                editCurfewEnd={state.editCurfewEnd}
                setEditCurfewEnd={state.setEditCurfewEnd}
                updating={state.updating}
                handleUpdate={state.handleUpdate}
            />
        </motion.div>
    );
};
export default UserToken;
