from . import views
from django.urls import path

urlpatterns = [
    path('phantom_test/', views.phantom_test, name='phantom_test'),
    path('nft_test/', views.nft_test, name='nft_test'),
    path('nft_mint/', views.nft_mint, name='nft_mint'),
    path('init_org_pool/', views.init_org_pool, name='init_org_pool'),
    path('add_org/', views.add_org, name='add_org'),
    path('remove_org/', views.remove_org, name='remove_org'),
    path('create_event/', views.create_event, name='create_event'),
    path('init_event_reg/', views.init_event_reg, name='init_event_reg'),
]