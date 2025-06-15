package spireclient

import (
	agentv1 "github.com/spiffe/spire-api-sdk/proto/spire/api/server/agent/v1"
	bundlev1 "github.com/spiffe/spire-api-sdk/proto/spire/api/server/bundle/v1"
	entryv1 "github.com/spiffe/spire-api-sdk/proto/spire/api/server/entry/v1"
	svidv1 "github.com/spiffe/spire-api-sdk/proto/spire/api/server/svid/v1"
	trustdomainv1 "github.com/spiffe/spire-api-sdk/proto/spire/api/server/trustdomain/v1"
)

// AgentClient returns the Agent service client
func (c *Client) AgentClient() agentv1.AgentClient {
	return agentv1.NewAgentClient(c.conn)
}

// BundleClient returns the Bundle service client
func (c *Client) BundleClient() bundlev1.BundleClient {
	return bundlev1.NewBundleClient(c.conn)
}

// EntryClient returns the Entry service client
func (c *Client) EntryClient() entryv1.EntryClient {
	return entryv1.NewEntryClient(c.conn)
}

// SVIDClient returns the SVID service client
func (c *Client) SVIDClient() svidv1.SVIDClient {
	return svidv1.NewSVIDClient(c.conn)
}

// TrustDomainClient returns the TrustDomain service client
func (c *Client) TrustDomainClient() trustdomainv1.TrustDomainClient {
	return trustdomainv1.NewTrustDomainClient(c.conn)
}
