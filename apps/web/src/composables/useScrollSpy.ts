import { onBeforeUnmount, onMounted, ref } from 'vue'

const SCROLL_OFFSET = 96

export function useScrollSpy(sectionIds: readonly string[]) {
  const activeId = ref<string | null>(sectionIds[0] ?? null)

  let observer: IntersectionObserver | null = null
  let scrollHandler: (() => void) | null = null

  function getElements(): HTMLElement[] {
    return sectionIds
      .map((id) => document.getElementById(id))
      .filter((element): element is HTMLElement => element !== null)
  }

  function pickActiveFromScroll(elements: HTMLElement[]) {
    let topmost: string | null = null

    for (const element of elements) {
      if (element.getBoundingClientRect().top <= SCROLL_OFFSET) {
        topmost = element.id
      }
    }

    activeId.value = topmost ?? sectionIds[0] ?? null
  }

  onMounted(() => {
    const elements = getElements()

    if (elements.length === 0) {
      return
    }

    if (typeof window.IntersectionObserver !== 'undefined') {
      observer = new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            if (entry.isIntersecting && sectionIds.includes(entry.target.id)) {
              activeId.value = entry.target.id
            }
          }
        },
        { rootMargin: '0px 0px -68% 0px' },
      )

      for (const element of elements) {
        observer.observe(element)
      }

      return
    }

    scrollHandler = () => pickActiveFromScroll(elements)
    window.addEventListener('scroll', scrollHandler, { passive: true })
    pickActiveFromScroll(elements)
  })

  onBeforeUnmount(() => {
    observer?.disconnect()
    observer = null

    if (scrollHandler) {
      window.removeEventListener('scroll', scrollHandler)
      scrollHandler = null
    }
  })

  return {
    activeId,
  }
}
