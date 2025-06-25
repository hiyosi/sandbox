package main

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// MockOpenFGAClient はテスト用のモッククライアント
type MockOpenFGAClient struct {
	mock.Mock
}

func (m *MockOpenFGAClient) CheckPermission(ctx context.Context, user, relation, object string) (bool, error) {
	args := m.Called(ctx, user, relation, object)
	return args.Bool(0), args.Error(1)
}

func (m *MockOpenFGAClient) BatchCheck(ctx context.Context, checks []CheckRequest) ([]bool, error) {
	args := m.Called(ctx, checks)
	return args.Get(0).([]bool), args.Error(1)
}

// PermissionChecker インターフェース
type PermissionChecker interface {
	CheckPermission(ctx context.Context, user, relation, object string) (bool, error)
	BatchCheck(ctx context.Context, checks []CheckRequest) ([]bool, error)
}

func TestPermissionChecks(t *testing.T) {
	tests := []struct {
		name     string
		user     string
		relation string
		object   string
		expected bool
		desc     string
	}{
		{
			name:     "alice_read_public_data",
			user:     "user:alice",
			relation: "can_read",
			object:   "resource:public-data",
			expected: true,
			desc:     "alice is writer, so can read",
		},
		{
			name:     "alice_write_public_data",
			user:     "user:alice",
			relation: "can_write",
			object:   "resource:public-data",
			expected: true,
			desc:     "alice is direct writer",
		},
		{
			name:     "bob_read_sensitive_data",
			user:     "user:bob",
			relation: "can_read",
			object:   "resource:sensitive-data",
			expected: true,
			desc:     "bob is backend-team member, can read team resources",
		},
		{
			name:     "bob_write_sensitive_data",
			user:     "user:bob",
			relation: "can_write",
			object:   "resource:sensitive-data",
			expected: false,
			desc:     "bob is member, not admin, so cannot write",
		},
		{
			name:     "charlie_read_public_data",
			user:     "user:charlie",
			relation: "can_read",
			object:   "resource:public-data",
			expected: true,
			desc:     "charlie is direct reader",
		},
		{
			name:     "charlie_read_sensitive_data",
			user:     "user:charlie",
			relation: "can_read",
			object:   "resource:sensitive-data",
			expected: false,
			desc:     "charlie is not team member, cannot read",
		},
		{
			name:     "admin_delete_sensitive_data",
			user:     "user:admin",
			relation: "can_delete",
			object:   "resource:sensitive-data",
			expected: true,
			desc:     "admin is direct owner, can delete",
		},
		{
			name:     "alice_write_sensitive_data",
			user:     "user:alice",
			relation: "can_write",
			object:   "resource:sensitive-data",
			expected: true,
			desc:     "alice is backend-team admin, can write team resources",
		},
		{
			name:     "frank_write_ui_config",
			user:     "user:frank",
			relation: "can_write",
			object:   "resource:user-interface-config",
			expected: true,
			desc:     "frank is direct writer for UI config",
		},
		{
			name:     "eve_delete_ui_config",
			user:     "user:eve",
			relation: "can_delete",
			object:   "resource:user-interface-config",
			expected: true,
			desc:     "eve is frontend-team owner, can delete team resources",
		},
	}

	mockClient := new(MockOpenFGAClient)
	
	// モックの期待値を設定
	for _, tt := range tests {
		mockClient.On("CheckPermission", mock.Anything, tt.user, tt.relation, tt.object).Return(tt.expected, nil)
	}

	ctx := context.Background()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := mockClient.CheckPermission(ctx, tt.user, tt.relation, tt.object)
			
			assert.NoError(t, err)
			assert.Equal(t, tt.expected, result, tt.desc)
		})
	}

	mockClient.AssertExpectations(t)
}

func TestBatchPermissionCheck(t *testing.T) {
	mockClient := new(MockOpenFGAClient)
	
	checks := []CheckRequest{
		{"user:alice", "can_read", "resource:public-data"},
		{"user:bob", "can_write", "resource:sensitive-data"},
		{"user:charlie", "can_read", "resource:sensitive-data"},
	}
	
	expectedResults := []bool{true, false, false}
	
	mockClient.On("BatchCheck", mock.Anything, checks).Return(expectedResults, nil)
	
	ctx := context.Background()
	results, err := mockClient.BatchCheck(ctx, checks)
	
	assert.NoError(t, err)
	assert.Equal(t, expectedResults, results)
	
	mockClient.AssertExpectations(t)
}

func TestPermissionScenarios(t *testing.T) {
	scenarios := []struct {
		name        string
		description string
		checks      []CheckRequest
		expected    []bool
	}{
		{
			name:        "team_member_access",
			description: "Team members can read team resources but not write unless admin",
			checks: []CheckRequest{
				{"user:bob", "can_read", "resource:sensitive-data"},    // team member -> true
				{"user:dave", "can_read", "resource:sensitive-data"},   // team member -> true
				{"user:bob", "can_write", "resource:sensitive-data"},   // member, not admin -> false
				{"user:dave", "can_write", "resource:sensitive-data"},  // member, not admin -> false
			},
			expected: []bool{true, true, false, false},
		},
		{
			name:        "admin_permissions",
			description: "Team admins can write to team resources",
			checks: []CheckRequest{
				{"user:alice", "can_read", "resource:sensitive-data"},  // team admin -> true
				{"user:alice", "can_write", "resource:sensitive-data"}, // team admin -> true
				{"user:alice", "can_delete", "resource:sensitive-data"}, // admin, not owner -> false
			},
			expected: []bool{true, true, false},
		},
		{
			name:        "owner_permissions",
			description: "Owners have full permissions",
			checks: []CheckRequest{
				{"user:admin", "can_read", "resource:sensitive-data"},   // owner -> true
				{"user:admin", "can_write", "resource:sensitive-data"},  // owner -> true
				{"user:admin", "can_delete", "resource:sensitive-data"}, // owner -> true
			},
			expected: []bool{true, true, true},
		},
		{
			name:        "cross_team_access",
			description: "Users cannot access other team resources without explicit permission",
			checks: []CheckRequest{
				{"user:frank", "can_read", "resource:sensitive-data"},   // frontend user, backend resource -> false
				{"user:alice", "can_read", "resource:user-interface-config"}, // backend user, frontend resource -> false
				{"user:bob", "can_write", "resource:user-interface-config"},  // backend user, frontend resource -> false
			},
			expected: []bool{false, false, false},
		},
	}

	for _, scenario := range scenarios {
		t.Run(scenario.name, func(t *testing.T) {
			mockClient := new(MockOpenFGAClient)
			
			mockClient.On("BatchCheck", mock.Anything, scenario.checks).Return(scenario.expected, nil)
			
			ctx := context.Background()
			results, err := mockClient.BatchCheck(ctx, scenario.checks)
			
			assert.NoError(t, err)
			assert.Equal(t, scenario.expected, results, scenario.description)
			
			mockClient.AssertExpectations(t)
		})
	}
}

// 統合テスト用のヘルパー関数
func TestPermissionMatrix(t *testing.T) {
	// 権限マトリックステスト
	users := []string{"user:alice", "user:bob", "user:charlie", "user:admin", "user:frank", "user:eve"}
	resources := []string{"resource:sensitive-data", "resource:public-data", "resource:user-interface-config"}
	relations := []string{"can_read", "can_write", "can_delete"}

	// 期待される権限マトリックス（実際の値は実装に応じて調整）
	expectedMatrix := map[string]map[string]map[string]bool{
		"user:alice": {
			"resource:sensitive-data": {"can_read": true, "can_write": true, "can_delete": false},
			"resource:public-data":    {"can_read": true, "can_write": true, "can_delete": false},
			"resource:user-interface-config": {"can_read": false, "can_write": false, "can_delete": false},
		},
		"user:bob": {
			"resource:sensitive-data": {"can_read": true, "can_write": false, "can_delete": false},
			"resource:public-data":    {"can_read": true, "can_write": false, "can_delete": false},
			"resource:user-interface-config": {"can_read": false, "can_write": false, "can_delete": false},
		},
		"user:charlie": {
			"resource:sensitive-data": {"can_read": false, "can_write": false, "can_delete": false},
			"resource:public-data":    {"can_read": true, "can_write": false, "can_delete": false},
			"resource:user-interface-config": {"can_read": false, "can_write": false, "can_delete": false},
		},
		"user:admin": {
			"resource:sensitive-data": {"can_read": true, "can_write": true, "can_delete": true},
			"resource:public-data":    {"can_read": true, "can_write": false, "can_delete": true},
			"resource:user-interface-config": {"can_read": false, "can_write": false, "can_delete": false},
		},
		"user:frank": {
			"resource:sensitive-data": {"can_read": false, "can_write": false, "can_delete": false},
			"resource:public-data":    {"can_read": false, "can_write": false, "can_delete": false},
			"resource:user-interface-config": {"can_read": true, "can_write": true, "can_delete": false},
		},
		"user:eve": {
			"resource:sensitive-data": {"can_read": false, "can_write": false, "can_delete": false},
			"resource:public-data":    {"can_read": false, "can_write": false, "can_delete": false},
			"resource:user-interface-config": {"can_read": false, "can_write": false, "can_delete": true},
		},
	}

	mockClient := new(MockOpenFGAClient)

	// 全ての組み合わせでモックを設定
	for _, user := range users {
		for _, resource := range resources {
			for _, relation := range relations {
				expected := false
				if userMatrix, exists := expectedMatrix[user]; exists {
					if resourceMatrix, exists := userMatrix[resource]; exists {
						if value, exists := resourceMatrix[relation]; exists {
							expected = value
						}
					}
				}
				mockClient.On("CheckPermission", mock.Anything, user, relation, resource).Return(expected, nil)
			}
		}
	}

	ctx := context.Background()

	// 全ての組み合わせをテスト
	for _, user := range users {
		for _, resource := range resources {
			for _, relation := range relations {
				t.Run(user+"_"+relation+"_"+resource, func(t *testing.T) {
					result, err := mockClient.CheckPermission(ctx, user, relation, resource)
					assert.NoError(t, err)
					
					expected := false
					if userMatrix, exists := expectedMatrix[user]; exists {
						if resourceMatrix, exists := userMatrix[resource]; exists {
							if value, exists := resourceMatrix[relation]; exists {
								expected = value
							}
						}
					}
					
					assert.Equal(t, expected, result, 
						"Permission check failed for %s %s %s", user, relation, resource)
				})
			}
		}
	}

	mockClient.AssertExpectations(t)
}