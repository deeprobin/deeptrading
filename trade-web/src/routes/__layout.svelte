<script lang="js">
  //import { Router, links, Route } from "svelte-routing";
  import "carbon-components-svelte/css/all.css";
  import {
    Header,
    HeaderNav,
    HeaderNavItem,
    HeaderNavMenu,
    HeaderUtilities,
    HeaderAction,
    HeaderPanelLink,
    HeaderPanelLinks,
    HeaderPanelDivider,
    SideNav,
    SideNavItems,
    SideNavMenu,
    SideNavMenuItem,
    SideNavLink,
    SideNavDivider,
    SkipToContent,
    Content,
    InlineNotification,
    Grid,
    Row,
    Column,
  } from "carbon-components-svelte";
  import Fade from "carbon-icons-svelte/lib/Fade.svelte";

  let isSideNavOpen = false;
  let isOpen = false;
  let transition = {
    duration: 200,
  };
</script>

<div>
    <Header company="DeepTrading" platformName="Dashboard" bind:isSideNavOpen>
      <svelte:fragment slot="skip-to-content">
        <SkipToContent />
      </svelte:fragment>
      <HeaderNav>
        <HeaderNavItem href="/stocks/" text="Stocks" />
        <HeaderNavItem href="/derivatives/" text="Derivatives" />
        <HeaderNavItem href="/forex/" text="Foreign Exchange" />
        <HeaderNavMenu text="Funds">
          <HeaderNavItem
            href="/funds/etfs"
            text="ETFs"
            alt="Exchange-traded fund"
          />
          <HeaderNavItem href="/funds/cefs" text="CEFs" alt="Closed-end fund" />
          <HeaderNavItem
            href="/funds/active-managed"
            text="Active managed funds"
          />
        </HeaderNavMenu>
        <HeaderNavMenu text="Tools">
          <HeaderNavItem href="/tools/inflation_visualizer/" text="Inflation Calculator" />
        </HeaderNavMenu>
        <HeaderNavItem href="/" text="Admin" />
      </HeaderNav>
      <HeaderUtilities>
        <HeaderAction bind:isOpen {transition}>
          <HeaderPanelLinks>
            <HeaderPanelDivider>Internal Apps</HeaderPanelDivider>
            <HeaderPanelLink>Grafana</HeaderPanelLink>
            <HeaderPanelLink>Prometheus</HeaderPanelLink>
            <HeaderPanelLink>InfluxDB</HeaderPanelLink>
            <HeaderPanelLink>PGAdmin</HeaderPanelLink>
            <HeaderPanelDivider>External Apps</HeaderPanelDivider>
            <HeaderPanelLink href="https://tradytics.com/">
              Tradytics
            </HeaderPanelLink>
            <HeaderPanelLink href="https://binance.com/">
              Binance
            </HeaderPanelLink>
          </HeaderPanelLinks>
        </HeaderAction>
      </HeaderUtilities>
      <SideNav bind:isOpen={isSideNavOpen} rail>
        <SideNavItems>
          <SideNavLink icon={Fade} text="Link 1" href="/" isSelected />
          <SideNavLink icon={Fade} text="Link 2" href="/" />
          <SideNavLink icon={Fade} text="Link 3" href="/" />
          <SideNavMenu icon={Fade} text="Menu">
            <SideNavMenuItem href="/" text="Link 1" />
            <SideNavMenuItem href="/" text="Link 2" />
            <SideNavMenuItem href="/" text="Link 3" />
          </SideNavMenu>
          <SideNavDivider />
          <SideNavLink icon={Fade} text="Link 4" href="/" />
        </SideNavItems>
      </SideNav>
    </Header>

    <Content>
		<slot/>
		<!--
      <Route path="/" component={Home} />
      <Route path="/inflation/" component={InflationVisualizer} />
      <Route path="/stock/" component={Stock} />
      <Route let:location>
        <InlineNotification
          hideCloseButton
          title="Error:"
          subtitle={`No route found for ${location.pathname}`}
        />
      </Route>-->
    </Content>
</div>
