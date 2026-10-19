import { useState, useRef, useCallback } from 'react';
import type { SyncInterval } from '../../../services/instanceService';
import { getPromptTreeSyncInterval } from '../../../services/instanceService';
import { getArchivedProjectsForInstance } from '../../../services/instanceService';
import { PromptViewMode, type AgmConversationNode, type AgmProjectTreeNode, type ViewMode } from './types';

export interface InspectorPrompt {
    title: string;
    text: string;
    convId: string;
    projectPath: string;
    images: string[];
    files: string[];
}

export interface PromptTreeState {
    treeData: AgmProjectTreeNode[];
    setTreeData: React.Dispatch<React.SetStateAction<AgmProjectTreeNode[]>>;
    isLoading: boolean;
    setIsLoading: React.Dispatch<React.SetStateAction<boolean>>;
    isAutoSyncing: boolean;
    setIsAutoSyncing: React.Dispatch<React.SetStateAction<boolean>>;
    syncInterval: SyncInterval;
    setSyncInterval: React.Dispatch<React.SetStateAction<SyncInterval>>;
    error: string | null;
    setError: React.Dispatch<React.SetStateAction<string | null>>;
    searchQuery: string;
    setSearchQuery: React.Dispatch<React.SetStateAction<string>>;
    activeFilter: 'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived';
    setActiveFilter: React.Dispatch<React.SetStateAction<'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived'>>;
    selectedFilter: 'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived';
    setSelectedFilter: React.Dispatch<React.SetStateAction<'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived'>>;
    categoryFilter: 'all' | 'user' | 'subagent' | 'system';
    setCategoryFilter: React.Dispatch<React.SetStateAction<'all' | 'user' | 'subagent' | 'system'>>;
    restoreFileInputRef: React.RefObject<HTMLInputElement | null>;
    isFullscreen: boolean;
    setIsFullscreen: React.Dispatch<React.SetStateAction<boolean>>;
    pinnedProjectIds: string[];
    setPinnedProjectIds: React.Dispatch<React.SetStateAction<string[]>>;
    archivedProjectIds: string[];
    setArchivedProjectIds: React.Dispatch<React.SetStateAction<string[]>>;
    isArchivedProjectsExpanded: boolean;
    setIsArchivedProjectsExpanded: React.Dispatch<React.SetStateAction<boolean>>;
    refreshingProjectId: string | null;
    setRefreshingProjectId: React.Dispatch<React.SetStateAction<string | null>>;
    confirmationSuffix: string;
    setConfirmationSuffix: React.Dispatch<React.SetStateAction<string>>;
    conversationTargetMode: 'current' | 'new';
    setConversationTargetMode: React.Dispatch<React.SetStateAction<'current' | 'new'>>;
    newConversationTitle: string;
    setNewConversationTitle: (title: string) => void;
    isCopiedText: boolean;
    setIsCopiedText: React.Dispatch<React.SetStateAction<boolean>>;
    isCopiedRaw: boolean;
    setIsCopiedRaw: React.Dispatch<React.SetStateAction<boolean>>;
    selectedProjectScope: string;
    setSelectedProjectScope: React.Dispatch<React.SetStateAction<string>>;
    isDetailsModalOpen: boolean;
    setIsDetailsModalOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isDetailsOpen: boolean;
    setIsDetailsOpen: React.Dispatch<React.SetStateAction<boolean>>;
    setIsExpandedFull: React.Dispatch<React.SetStateAction<boolean>>;
    activeTab: 'instruction' | 'results';
    setActiveTab: React.Dispatch<React.SetStateAction<'instruction' | 'results'>>;
    isRerunMenuOpen: boolean;
    setIsRerunMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isFocusing: boolean;
    setIsFocusing: React.Dispatch<React.SetStateAction<boolean>>;
    copiedField: string | null;
    setCopiedField: React.Dispatch<React.SetStateAction<string | null>>;
    expandedProjects: Record<string, boolean>;
    setExpandedProjects: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
    expandedStaleGroups: Record<string, boolean>;
    setExpandedStaleGroups: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
    selectedProject: AgmProjectTreeNode | null;
    setSelectedProject: React.Dispatch<React.SetStateAction<AgmProjectTreeNode | null>>;
    selectedConversation: AgmConversationNode | null;
    setSelectedConversation: React.Dispatch<React.SetStateAction<AgmConversationNode | null>>;
    activePromptText: string;
    setActivePromptText: React.Dispatch<React.SetStateAction<string>>;
    editedPromptText: string;
    setEditedPromptText: React.Dispatch<React.SetStateAction<string>>;
    viewMode: ViewMode;
    setViewMode: React.Dispatch<React.SetStateAction<ViewMode>>;
    showAllWords: boolean;
    setShowAllWords: React.Dispatch<React.SetStateAction<boolean>>;
    previewTab: 'instruction' | 'results';
    setPreviewTab: React.Dispatch<React.SetStateAction<'instruction' | 'results'>>;
    isCopiedResults: boolean;
    setIsCopiedResults: React.Dispatch<React.SetStateAction<boolean>>;
    isCopied: boolean;
    setIsCopied: React.Dispatch<React.SetStateAction<boolean>>;
    isResending: boolean;
    setIsResending: React.Dispatch<React.SetStateAction<boolean>>;
    isEnqueueing: boolean;
    setIsEnqueueing: React.Dispatch<React.SetStateAction<boolean>>;
    isCopyMenuOpen: boolean;
    setIsCopyMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isSendMenuOpen: boolean;
    setIsSendMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isEnqueueMenuOpen: boolean;
    setIsEnqueueMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isCopiedAll: boolean;
    setIsCopiedAll: React.Dispatch<React.SetStateAction<boolean>>;
    isExportMenuOpen: boolean;
    setIsExportMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    actionMsg: string | null;
    setActionMsg: React.Dispatch<React.SetStateAction<string | null>>;
    selectedConversationRef: React.MutableRefObject<AgmConversationNode | null>;
    selectedProjectRef: React.MutableRefObject<AgmProjectTreeNode | null>;
    activePromptTextRef: React.MutableRefObject<string>;
    elapsedSeconds: number;
    setElapsedSeconds: React.Dispatch<React.SetStateAction<number>>;
    inspectorPrompt: InspectorPrompt | null;
    setInspectorPrompt: React.Dispatch<React.SetStateAction<InspectorPrompt | null>>;
    isTreeCollapsed: boolean;
    setIsTreeCollapsed: React.Dispatch<React.SetStateAction<boolean>>;
    isAutomatedHidden: boolean;
    setIsAutomatedHidden: React.Dispatch<React.SetStateAction<boolean>>;
}

export function usePromptTreeState(instanceId: string): PromptTreeState {
    const [treeData, setTreeData] = useState<AgmProjectTreeNode[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [isAutoSyncing, setIsAutoSyncing] = useState(false);
    const [syncInterval, setSyncInterval] = useState<SyncInterval>(() => getPromptTreeSyncInterval());
    const [error, setError] = useState<string | null>(null);
    const [searchQuery, _setSearchQuery] = useState('');
    const setSearchQuery = useCallback((val: React.SetStateAction<string>) => {
        _setSearchQuery(val);
    }, []);

    const [activeFilter, _setActiveFilter] = useState<'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived'>('all');
    const setActiveFilter = useCallback((val: React.SetStateAction<'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived'>) => {
        _setActiveFilter(val);
    }, []);
    const setSelectedFilter = setActiveFilter;

    const [categoryFilter, setCategoryFilter] = useState<'all' | 'user' | 'subagent' | 'system'>('all');
    const [isTreeCollapsed, setIsTreeCollapsed] = useState<boolean>(true);
    const [isAutomatedHidden, setIsAutomatedHidden] = useState<boolean>(true);
    const restoreFileInputRef = useRef<HTMLInputElement>(null);
    const [isFullscreen, setIsFullscreen] = useState(false);

    // Pinned projects persisted in localStorage
    const [pinnedProjectIds, setPinnedProjectIds] = useState<string[]>(() => {
        try {
            const stored = localStorage.getItem(`agm_pinned_projects_${instanceId || 'default'}`);
            return stored ? JSON.parse(stored) : [];
        } catch {
            return [];
        }
    });

    // Archived projects persisted in localStorage
    const [archivedProjectIds, setArchivedProjectIds] = useState<string[]>(() => {
        return getArchivedProjectsForInstance(instanceId);
    });

    const [isArchivedProjectsExpanded, setIsArchivedProjectsExpanded] = useState(false);

    const [refreshingProjectId, setRefreshingProjectId] = useState<string | null>(null);
    const [confirmationSuffix, setConfirmationSuffix] = useState<string>('None (Send as is)');
    const [conversationTargetMode, setConversationTargetMode] = useState<'current' | 'new'>('current');
    const [newConversationTitle, setNewConversationTitle] = useState<string>('');
    const [isCopiedText, setIsCopiedText] = useState(false);
    const [isCopiedRaw, setIsCopiedRaw] = useState(false);

    const [selectedProjectScope, setSelectedProjectScope] = useState<string>('all');
    const [isDetailsModalOpen, _setIsDetailsModalOpen] = useState(false);
    const setIsDetailsModalOpen = useCallback((val: React.SetStateAction<boolean>) => {
        _setIsDetailsModalOpen(val);
    }, []);
    const setIsDetailsOpen = setIsDetailsModalOpen;

    const [isFocusing, setIsFocusing] = useState(false);
    const [copiedField, setCopiedField] = useState<string | null>(null);

    const [expandedProjects, setExpandedProjects] = useState<Record<string, boolean>>({});
    const [expandedStaleGroups, setExpandedStaleGroups] = useState<Record<string, boolean>>({});

    const [selectedProject, setSelectedProject] = useState<AgmProjectTreeNode | null>(null);
    const [selectedConversation, setSelectedConversation] = useState<AgmConversationNode | null>(null);
    const [activePromptText, setActivePromptText] = useState<string>('');
    const [editedPromptText, setEditedPromptText] = useState<string>('');
    const [viewMode, setViewMode] = useState<ViewMode>(PromptViewMode.Preview);
    const [showAllWords, _setShowAllWords] = useState<boolean>(() => {
        try {
            return localStorage.getItem('agm_prompt_tree_show_all_words') === 'true';
        } catch {
            return false;
        }
    });
    const setShowAllWords: React.Dispatch<React.SetStateAction<boolean>> = useCallback((val) => {
        _setShowAllWords((prev) => {
            const next = typeof val === 'function' ? val(prev) : val;
            try {
                localStorage.setItem('agm_prompt_tree_show_all_words', String(next));
            } catch {}
            return next;
        });
    }, []);
    const setIsExpandedFull = setShowAllWords;

    const [previewTab, _setPreviewTab] = useState<'instruction' | 'results'>('instruction');
    const setPreviewTab = useCallback((val: React.SetStateAction<'instruction' | 'results'>) => {
        _setPreviewTab(val);
    }, []);
    const setActiveTab = setPreviewTab;

    const [isCopiedResults, setIsCopiedResults] = useState(false);

    const [isCopied, setIsCopied] = useState(false);
    const [isCopiedAll, setIsCopiedAll] = useState(false);
    const [isResending, setIsResending] = useState(false);
    const [isEnqueueing, setIsEnqueueing] = useState(false);
    const [isCopyMenuOpen, _setIsCopyMenuOpen] = useState(false);
    const [isSendMenuOpen, _setIsSendMenuOpen] = useState(false);
    const [isEnqueueMenuOpen, _setIsEnqueueMenuOpen] = useState(false);
    const [isRerunMenuOpen, _setIsRerunMenuOpen] = useState(false);
    const [actionMsg, setActionMsg] = useState<string | null>(null);

    // Single-active menu invariant: opening one menu closes the other menus
    const setIsCopyMenuOpen: React.Dispatch<React.SetStateAction<boolean>> = useCallback((val) => {
        _setIsCopyMenuOpen((prev) => {
            const next = typeof val === 'function' ? val(prev) : val;
            if (next) {
                _setIsSendMenuOpen(false);
                _setIsEnqueueMenuOpen(false);
                _setIsRerunMenuOpen(false);
            }
            return next;
        });
    }, []);

    const setIsSendMenuOpen: React.Dispatch<React.SetStateAction<boolean>> = useCallback((val) => {
        _setIsSendMenuOpen((prev) => {
            const next = typeof val === 'function' ? val(prev) : val;
            if (next) {
                _setIsCopyMenuOpen(false);
                _setIsEnqueueMenuOpen(false);
                _setIsRerunMenuOpen(false);
            }
            return next;
        });
    }, []);

    const setIsEnqueueMenuOpen: React.Dispatch<React.SetStateAction<boolean>> = useCallback((val) => {
        _setIsEnqueueMenuOpen((prev) => {
            const next = typeof val === 'function' ? val(prev) : val;
            if (next) {
                _setIsCopyMenuOpen(false);
                _setIsSendMenuOpen(false);
                _setIsRerunMenuOpen(false);
            }
            return next;
        });
    }, []);

    const setIsRerunMenuOpen: React.Dispatch<React.SetStateAction<boolean>> = useCallback((val) => {
        _setIsRerunMenuOpen((prev) => {
            const next = typeof val === 'function' ? val(prev) : val;
            if (next) {
                _setIsCopyMenuOpen(false);
                _setIsSendMenuOpen(false);
                _setIsEnqueueMenuOpen(false);
            }
            return next;
        });
    }, []);

    const setIsExportMenuOpen = setIsCopyMenuOpen;

    // Live refs to prevent closure staleness in background auto-sync timer
    const selectedConversationRef = useRef(selectedConversation);
    selectedConversationRef.current = selectedConversation;
    const selectedProjectRef = useRef(selectedProject);
    selectedProjectRef.current = selectedProject;
    const activePromptTextRef = useRef(activePromptText);
    activePromptTextRef.current = activePromptText;

    const [elapsedSeconds, setElapsedSeconds] = useState(0);

    // Full screen prompt detail inspector
    const [inspectorPrompt, setInspectorPrompt] = useState<InspectorPrompt | null>(null);

    return {
        treeData, setTreeData,
        isLoading, setIsLoading,
        isAutoSyncing, setIsAutoSyncing,
        syncInterval, setSyncInterval,
        error, setError,
        searchQuery, setSearchQuery,
        activeFilter, setActiveFilter,
        categoryFilter, setCategoryFilter,
        restoreFileInputRef,
        isFullscreen, setIsFullscreen,
        pinnedProjectIds, setPinnedProjectIds,
        archivedProjectIds, setArchivedProjectIds,
        isArchivedProjectsExpanded, setIsArchivedProjectsExpanded,
        refreshingProjectId, setRefreshingProjectId,
        confirmationSuffix, setConfirmationSuffix,
        conversationTargetMode, setConversationTargetMode,
        newConversationTitle, setNewConversationTitle,
        isCopiedText, setIsCopiedText,
        isCopiedRaw, setIsCopiedRaw,
        selectedProjectScope, setSelectedProjectScope,
        isDetailsModalOpen, setIsDetailsModalOpen,
        isFocusing, setIsFocusing,
        copiedField, setCopiedField,
        expandedProjects, setExpandedProjects,
        expandedStaleGroups, setExpandedStaleGroups,
        selectedProject, setSelectedProject,
        selectedConversation, setSelectedConversation,
        activePromptText, setActivePromptText,
        editedPromptText, setEditedPromptText,
        viewMode, setViewMode,
        showAllWords, setShowAllWords,
        previewTab, setPreviewTab,
        isCopiedResults, setIsCopiedResults,
        isCopied, setIsCopied,
        isCopiedAll, setIsCopiedAll,
        isResending, setIsResending,
        isEnqueueing, setIsEnqueueing,
        isCopyMenuOpen, setIsCopyMenuOpen,
        isSendMenuOpen, setIsSendMenuOpen,
        isEnqueueMenuOpen, setIsEnqueueMenuOpen,
        isExportMenuOpen: isCopyMenuOpen, setIsExportMenuOpen: setIsCopyMenuOpen,
        actionMsg, setActionMsg,
        selectedConversationRef,
        selectedProjectRef,
        activePromptTextRef,
        elapsedSeconds, setElapsedSeconds,
        inspectorPrompt, setInspectorPrompt,
        isTreeCollapsed, setIsTreeCollapsed,
        isAutomatedHidden, setIsAutomatedHidden,
    };
}
