'use strict'

const m = require('mithril')
const modals = require('App/components/modals')

const _navLink = (route, asset_active, asset_inactive, label) =>
  m('li.nav-item.retailer_nav',
    m(`a.nav-link.retailer_nav_link[href=${route}]`,
      { class: m.route.get().startsWith(route) ? 'active' : '', oncreate: m.route.link },
      [m(`img.nav_icon[src=/assets/images/${m.route.get().startsWith(route) ? asset_active : asset_inactive}]`), m('span.nav_label.p-1.ml-1', label)]))


const _greeting = () =>
  'Welcome, retail or brand member!'

const App = {
  _viewName: 'App',
  oninit: (vnode) => {
    vnode.state.loading = false
  },
  view: (vnode) => {
    if (vnode.state.loading) {
      return [m('.row', 'Loading...')]
    } else {
      return [
        m('nav.navbar.navbar-expand-md.navbar-light.bg-light',
          [
            m(
              "a.navbar-brand.org-brand.greeting_text[href=/]",
              { oncreate: m.route.link },
              [
                m(
                  "span.logo-circle",
                  m(
                    'img.org-logo[src="/assets/images/granted-certifications-active.svg"].d-inline-block.align-top'
                  )
                )
              ]
            ),
            m('span.ml-3.greeting_text', _greeting()),
            m('div.collapse.navbar-collapse', [
              m('ul.navbar-nav.ml-auto',
                [
                  _navLink('/certifications', 'certified-factories-icon.svg', 'inactive-cert-factories.svg', 'Certified Factories'),
                  _navLink('/agents', 'active-agents.svg', 'inactive-agents.svg', 'Agents'),
                ]),

            ])
          ]),
        m('main.container.mt-5', { role: 'main' }, [vnode.children]),
        m(modals.ModalContainer, { show: modals.displayModal() })
      ]
    }
  },

  subpage: (element) => ({
    onmatch: (_args, _requestedPath) => element,
    render: (vnode) => m(App, vnode)
  })
}

const Welcome = {
  _viewName: 'Welcome',
  view: () =>
    [
      m('div.landing-page.landing-page-retailer', [
        m('div.landing-page-info', [
          m('p.landing-page-info-section.landing-page-info-header', [
            'Use ',
            m('strong', 'ConsenSource'),
            ' to help retailers and brands ensure a transparent and responsible supply chain, from source to shelf'
          ]),
          m('ul.landing-page-info-section', [
            m('li', 'Search for a variety of certified factories'),
            m('li', 'View important certifications, licenses, and contact information for each retailer'),
            m('li', 'Rest assured that both past and current data are accurate, verified, and up-to-date')
          ]),
          m('button.btn.landing-page-action-btn', 'Start the search for certified factories')
        ])
      ])
    ],
}
module.exports = {
  App, Welcome
}
